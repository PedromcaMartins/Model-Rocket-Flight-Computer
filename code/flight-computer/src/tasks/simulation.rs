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

use postcard_rpc::server::{Dispatch, Server, WireRx, WireTx};
use proto::{SimAltimeterLedTopic, SimArmLedTopic, SimDeploymentLedTopic, SimFileSystemLedTopic, SimGpsLedTopic, SimGroundStationLedTopic, SimImuLedTopic, SimPostcardLedTopic};

use crate::{interfaces::{FileSystem, impls::simulation::{sensor::{SimAltimeter, SimGps, SimImu}, arming_system::SimArming, deployment_system::SimRecovery, led::SimLed}}, tasks::{finite_state_machine_task, groundstation_task, postcard_server_task, run_flight_computer, sensor_task, storage_task}};

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
    ).await;
}

/// HOST entry point — two servers, [`HostFileSystem`] from [`HostConfig::default`].
///
/// `sim_server` carries the simulator peripheral surface (`fc-sim.sock`); all peripheral
/// instances use its sender. `gs_server` carries telemetry and commands (`fc-gs.sock`);
/// [`groundstation_task`] uses its sender. Both [`postcard_server_task`] futures are
/// composed with `join` before being passed to [`run_flight_computer`].
///
/// Called by `flight-computer-host::main` after binding both sockets. See
/// `flight-computer-host::dispatch` for the dispatch types wired to each server.
#[cfg(feature = "impl_host")]
#[inline]
pub async fn start_host_flight_computer<
    SimTx, SimRx, SimBuf, SimD,
    GsTx, GsRx, GsBuf, GsD,
>(
    sim_server: Server<SimTx, SimRx, SimBuf, SimD>,
    gs_server: Server<GsTx, GsRx, GsBuf, GsD>,
)
where
    SimTx: WireTx + Clone,
    SimRx: WireRx,
    SimBuf: DerefMut<Target = [u8]>,
    SimD: Dispatch<Tx = SimTx>,
    GsTx: WireTx + Clone,
    GsRx: WireRx,
    GsBuf: DerefMut<Target = [u8]>,
    GsD: Dispatch<Tx = GsTx>,
{
    use crate::config::host::HostConfig;
    use crate::interfaces::impls::host::filesystem::HostFileSystem;
    use embassy_futures::join::join;

    let dir_path = HostConfig::default();
    let filesystem = HostFileSystem::new(dir_path.storage_path).await;

    let sim_sender = sim_server.sender();
    let gs_sender = gs_server.sender();

    let postcard_sim_task = postcard_server_task(
        sim_server,
        SimLed::<_, SimPostcardLedTopic>::new(&sim_sender),
    );
    let postcard_gs_task = postcard_server_task(
        gs_server,
        SimLed::<_, SimPostcardLedTopic>::new(&gs_sender),
    );
    let postcard_task = join(postcard_sim_task, postcard_gs_task);

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

    let groundstation_task = groundstation_task(
        &gs_sender,
        SimLed::<_, SimGroundStationLedTopic>::new(&gs_sender),
    );

    run_flight_computer(
        finite_state_machine_task,
        storage_task,
        postcard_task,
        altimeter_task,
        gps_task,
        imu_task,
        groundstation_task,
    )
    .await;
}
