//! Entry points for simulator-fed FC deployments.
//!
//! Provides factory functions that wire peripheral implementations to simulator clients
//! and call [`run_flight_computer`] with the constructed task set.
//!
//! | Entry point | Mode | Transport |
//! |---|---|---|
//! | [`start_pil_flight_computer`] | PIL — FC on prod MCU, sim on host | One `Server` over USB |
//! | [`start_host_flight_computer`] | HOST — FC and sim as separate processes | Two `Server`s: `fc-sim.sock` + `fc-gs.sock` |
//!
//! `start_host_flight_computer` is called by the `flight-computer-host` binary.
//! See `flight-computer-host/src/main.rs` and `flight-computer-host/src/dispatch.rs`
//! for the dispatch types, socket binding, and startup sequence.

use core::ops::DerefMut;
use core::sync::atomic::{AtomicU32, Ordering};

use postcard_rpc::header::{VarHeader, VarSeq};
use postcard_rpc::server::{Dispatch, Server, Sender, WireRx, WireTx};
use proto::{
    actuator_data::ActuatorStatus,
    sensor_data::{AltimeterData, GpsData, ImuData},
};
use proto::wire::{
    RecordData, SimAltimeterLedTopic, SimArmLedTopic, SimDeploymentLedTopic,
    SimFileSystemLedTopic, SimFlightStateTopic, SimGpsLedTopic, SimGroundStationLedTopic,
    SimImuLedTopic, SimPostcardLedTopic,
};

use crate::{
    interfaces::{
        impls::simulation::{
            arming_system::SimArming, deployment_system::SimRecovery, led::SimLed,
            sensor::{SimAltimeter, SimGps, SimImu, SimSensor},
        },
        FileSystem, Led,
    },
    log::{error, warn},
    sync::FLIGHT_STATE_WATCH,
    tasks::{
        finite_state_machine_task, groundstation_task, postcard_server_task,
        run_flight_computer, sensor_task, storage_task,
    },
};

use super::postcard::Context;

// ---------------------------------------------------------------------------
// Sim dispatch handlers — called by the postcard-rpc server when the simulator
// publishes sensor/actuator data over the fc-sim socket (HOST) or USB (PIL).
// ---------------------------------------------------------------------------

pub fn sim_altimeter_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: AltimeterData, _out: &Sender<Tx>) {
    SimAltimeter::update_data(data);
}

pub fn sim_gps_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: GpsData, _out: &Sender<Tx>) {
    SimGps::update_data(data);
}

pub fn sim_imu_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: ImuData, _out: &Sender<Tx>) {
    SimImu::update_data(data);
}

pub fn sim_arming_activate<Tx: WireTx>(_context: &mut Context, _header: VarHeader, _data: ActuatorStatus, _out: &Sender<Tx>) {
    SimArming::activate();
}

/// Handles the server management for the fc-sim socket.
/// Panics on any disconnect — FC <-> simulator desync is unrecoverable.
pub async fn postcard_sim_server_task<Tx, Rx, Buf, D, LED>(
    mut server: Server<Tx, Rx, Buf, D>,
    mut led: LED,
) -> !
where
    Tx: postcard_rpc::server::WireTx,
    Rx: postcard_rpc::server::WireRx,
    Buf: DerefMut<Target = [u8]>,
    D: postcard_rpc::server::Dispatch<Tx = Tx>,
    LED: Led,
{
    led.on().await.unwrap_or_else(|e| warn!("Postcard sim server: Status Led error: {:?}", e));
    // `ServerError` may lack Debug/Display in no_std context; just log the exit.
    let _ = server.run().await;
    error!("sim server: run exited (connection dropped)");
    led.off().await.unwrap_or_else(|e| warn!("Postcard sim server: Status Led error: {:?}", e));
    panic!("fc-sim connection closed: FC and simulator desynced");
}

static UID_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Publishes `FlightState` changes on `SimFlightStateTopic` for the simulator.
#[inline]
pub async fn flight_state_sim_publisher_task<Tx>(postcard_sender: &Sender<Tx>)
where
    Tx: WireTx,
{
    let mut flight_state_receiver = FLIGHT_STATE_WATCH
        .receiver()
        .expect("Not enough flight state consumers");
    loop {
        let record = flight_state_receiver.changed().await;
        if let RecordData::FlightState(state) = record.payload().clone()
            && postcard_sender
                .publish::<SimFlightStateTopic>(
                    VarSeq::Seq4(UID_COUNTER.fetch_add(1, Ordering::Relaxed)),
                    &state,
                )
                .await
                .is_err()
        {
            warn!("flight_state_sim_publisher: publish failed");
        }
    }
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

/// PIL entry point — single server, caller-supplied filesystem.
///
/// All peripheral instances and the groundstation task share `server.sender()`.
/// `filesystem` is typically SD/flash in PIL.
#[inline]
pub async fn start_pil_flight_computer<
    FS,
    PostcardTx,
    PostcardRx,
    PostcardBuf,
    PostcardD,
> (
    filesystem: FS, 
    gs_server: Server<PostcardTx, PostcardRx, PostcardBuf, PostcardD>,
)
where 
    FS: FileSystem,
    PostcardTx: WireTx + Clone,
    PostcardRx: WireRx,
    PostcardBuf: DerefMut<Target = [u8]>,
    PostcardD: Dispatch<Tx = PostcardTx>,
{
    let postcard_sender = gs_server.sender();

    let postcard_task = postcard_server_task(
        gs_server,
        SimLed::<_, SimPostcardLedTopic>::new(&postcard_sender),
    );

    let altimeter_task = sensor_task(
        SimAltimeter,
        SimLed::<_, SimAltimeterLedTopic>::new(&postcard_sender),
    );
    let gps_task = sensor_task(
        SimGps,
        SimLed::<_, SimGpsLedTopic>::new(&postcard_sender),
    );
    let imu_task = sensor_task(
        SimImu,
        SimLed::<_, SimImuLedTopic>::new(&postcard_sender),
    );

    let finite_state_machine_task = finite_state_machine_task(
        SimArming, 
        SimLed::<_, SimArmLedTopic>::new(&postcard_sender),
        SimRecovery::new(&postcard_sender), 
        SimLed::<_, SimDeploymentLedTopic>::new(&postcard_sender),
    );

    let storage_task = storage_task(
        filesystem, 
        SimLed::<_, SimFileSystemLedTopic>::new(&postcard_sender), 
    );

    let groundstation_task = groundstation_task(
        &postcard_sender,
        SimLed::<_, SimGroundStationLedTopic>::new(&postcard_sender),
    );

    run_flight_computer(
        finite_state_machine_task,
        storage_task,
        postcard_task,
        altimeter_task,
        gps_task,
        imu_task,
        groundstation_task,
        flight_state_sim_publisher_task(&postcard_sender),
    ).await;
}

/// HOST entry point — sim server now, GS server wired in late.
///
/// `sim_server` carries the simulator peripheral surface (`fc-sim.sock`) and is
/// required up front: all peripheral instances use its sender and the FC starts
/// driving the FSM as soon as the simulator connects.
///
/// `gs_server_factory` builds a fresh GS server (`fc-gs.sock`) per call. The
/// deferred GS subsystem holds the factory and runs an accept -> serve -> reconnect
/// loop: a missing GS never blocks the FC <-> simulator loop, and a transient
/// GS disconnect/restart is recovered transparently. This matches M2.2: a
/// full HOST scenario runs with the GS process present, absent, or restarting.
///
/// Each call to `gs_server_factory` must return a freshly accepted `Server`;
/// it is invoked once per (re)connect attempt and must internally retry until
/// it has one (callers should log + retry on transient accept errors, not panic).
///
/// Called by `flight-computer-host::main`. See `flight-computer-host::dispatch`
/// for the dispatch types wired to each server.
#[cfg(feature = "impl_host")]
#[inline]
pub async fn start_host_flight_computer<
    SimTx, SimRx, SimBuf, SimD,
    F, GsFut, GsTx, GsRx, GsBuf, GsD,
>(
    sim_server: Server<SimTx, SimRx, SimBuf, SimD>,
    mut gs_server_factory: F,
)
where
    SimTx: WireTx + Clone,
    SimRx: WireRx,
    SimBuf: DerefMut<Target = [u8]>,
    SimD: Dispatch<Tx = SimTx>,
    F: FnMut() -> GsFut,
    GsFut: core::future::Future<Output = Server<GsTx, GsRx, GsBuf, GsD>>,
    GsTx: WireTx + Clone,
    GsRx: WireRx,
    GsBuf: DerefMut<Target = [u8]>,
    GsD: Dispatch<Tx = GsTx>,
{
    use crate::config::{host::HostConfig, PostcardConfig};
    use crate::interfaces::impls::host::filesystem::HostFileSystem;
    use embassy_futures::select::select;
    use embassy_time::Timer;

    let dir_path = std::path::PathBuf::from(HostConfig::STORAGE_PATH);
    let filesystem = HostFileSystem::new(dir_path).await;

    let sim_sender = sim_server.sender();

    let postcard_sim_task = postcard_sim_server_task(
        sim_server,
        SimLed::<_, SimPostcardLedTopic>::new(&sim_sender),
    );

    let altimeter_task = sensor_task(
        SimAltimeter,
        SimLed::<_, SimAltimeterLedTopic>::new(&sim_sender),
    );
    let gps_task = sensor_task(
        SimGps,
        SimLed::<_, SimGpsLedTopic>::new(&sim_sender),
    );
    let imu_task = sensor_task(
        SimImu,
        SimLed::<_, SimImuLedTopic>::new(&sim_sender),
    );

    let finite_state_machine_task = finite_state_machine_task(
        SimArming,
        SimLed::<_, SimArmLedTopic>::new(&sim_sender),
        SimRecovery::new(&sim_sender),
        SimLed::<_, SimDeploymentLedTopic>::new(&sim_sender),
    );

    let storage_task = storage_task(
        filesystem,
        SimLed::<_, SimFileSystemLedTopic>::new(&sim_sender),
    );

    // Deferred GS subsystem: accept -> serve -> reconnect.
    //
    // Each iteration:
    //   1. `gs_server_factory()` resolves once the GS backend (re)connects.
    //   2. The one-shot postcard server runs until the wire drops, raced
    //      against the telemetry relay (`groundstation_task`). When the
    //      postcard side ends, `select` drops the relay automatically.
    //   3. Sleep `RECONNECT_INTERVAL` to absorb GS-restart churn, then loop.
    //
    // The race is necessary because `groundstation_task` is `-> !` and would
    // otherwise hold the wire after the postcard server returned.
    let gs_subsystem = async move {
        loop {
            let gs_server = gs_server_factory().await;
            let gs_sender = gs_server.sender();
            let postcard_gs_task = crate::tasks::postcard_server_task_oneshot(
                gs_server,
                SimLed::<_, SimPostcardLedTopic>::new(&gs_sender),
            );
            let gs_relay = groundstation_task(
                &gs_sender,
                SimLed::<_, SimGroundStationLedTopic>::new(&gs_sender),
            );
            select(postcard_gs_task, gs_relay).await;
            crate::log::debug!("GS subsystem disconnected, waiting for reconnect...");
            Timer::after(PostcardConfig::RECONNECT_INTERVAL).await;
        }
    };

    run_flight_computer(
        finite_state_machine_task,
        storage_task,
        postcard_sim_task,
        altimeter_task,
        gps_task,
        imu_task,
        gs_subsystem,
        flight_state_sim_publisher_task(&sim_sender),
    )
    .await;
}
