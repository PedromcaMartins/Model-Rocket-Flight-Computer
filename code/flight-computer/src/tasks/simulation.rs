use core::ops::DerefMut;

use defmt_or_log::info;
use embassy_futures::select::{Either, Either6, select, select6};
use postcard_rpc::server::{Dispatch, Sender, Server, WireRx, WireTx};
use proto::{SimAltimeterLedTopic, SimArmLedTopic, SimDeploymentLedTopic, SimFileSystemLedTopic, SimGpsLedTopic, SimGroundStationLedTopic, SimImuLedTopic, SimPostcardLedTopic};

use crate::{interfaces::{FileSystem, impls::simulation::{sensor::{SimAltimeter, SimGps, SimImu}, arming_system::SimArming, deployment_system::SimRecovery, led::SimLed}}, tasks::{finite_state_machine_task, groundstation_task, postcard_server_task, sensor_task, storage_task}};

#[cfg(feature = "impl_host")]
pub async fn start_sil_flight_computer<
    PostcardTx,
    PostcardRx,
    PostcardBuf,
    PostcardD,
> (
    postcard_sender: Sender<PostcardTx>,
    server: Server<PostcardTx, PostcardRx, PostcardBuf, PostcardD>,
)
where 
    PostcardTx: WireTx + Clone,
    PostcardRx: WireRx,
    PostcardBuf: DerefMut<Target = [u8]>,
    PostcardD: Dispatch<Tx = PostcardTx>,
{
    use crate::{config::host::HostConfig, interfaces::impls::host::filesystem::HostFileSystem};
    let dir_path = HostConfig::default();

    start_pil_flight_computer(
        HostFileSystem::new(dir_path.storage_path).await,
        postcard_sender,
        server,
    ).await;
}

pub async fn start_pil_flight_computer<
    SdCard,
    PostcardTx,
    PostcardRx,
    PostcardBuf,
    PostcardD,
> (
    sd_card: SdCard, 
    postcard_sender: Sender<PostcardTx>,
    server: Server<PostcardTx, PostcardRx, PostcardBuf, PostcardD>,
)
where 
    SdCard: FileSystem,
    PostcardTx: WireTx + Clone,
    PostcardRx: WireRx,
    PostcardBuf: DerefMut<Target = [u8]>,
    PostcardD: Dispatch<Tx = PostcardTx>,
{
    let postcard_task = postcard_server_task(
        server,
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
        sd_card, 
        SimLed::<_, SimFileSystemLedTopic>::new(&postcard_sender), 
    );

    let groundstation_task = groundstation_task(
        &postcard_sender,
        SimLed::<_, SimGroundStationLedTopic>::new(&postcard_sender),
    );

    #[allow(clippy::ignored_unit_patterns)]
    match select(
        select6(
            postcard_task,
            altimeter_task, 
            gps_task, 
            imu_task,
            finite_state_machine_task, 
            storage_task,
        ),
        groundstation_task,
    ).await {
        Either::First(Either6::First(_))  => { info!("Postcard Server task ended") },
        Either::First(Either6::Second(_)) => { info!("Altimeter task ended") },
        Either::First(Either6::Third(_))  => { info!("GPS task ended") },
        Either::First(Either6::Fourth(_)) => { info!("IMU task ended") },
        Either::First(Either6::Fifth(_))  => { info!("Finite State Machine task ended") },
        Either::First(Either6::Sixth(_))  => { info!("Storage task ended") },
        Either::Second(_)                 => { info!("Groundstation task ended") },
    }

    info!("Flight Computer finished!");
}
