use core::ops::DerefMut;

use postcard_rpc::server::{Dispatch, Server, WireRx, WireTx};
use proto::{SimAltimeterLedTopic, SimArmLedTopic, SimDeploymentLedTopic, SimFileSystemLedTopic, SimGpsLedTopic, SimGroundStationLedTopic, SimImuLedTopic, SimPostcardLedTopic};

use crate::{interfaces::{FileSystem, impls::simulation::{sensor::{SimAltimeter, SimGps, SimImu}, arming_system::SimArming, deployment_system::SimRecovery, led::SimLed}}, tasks::{finite_state_machine_task, groundstation_task, postcard_server_task, run_flight_computer, sensor_task, storage_task}};

#[cfg(feature = "impl_host")]
#[inline]
pub async fn start_sil_flight_computer<
    PostcardTx,
    PostcardRx,
    PostcardBuf,
    PostcardD,
> (
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
        server,
    ).await;
}

#[inline]
pub async fn start_pil_flight_computer<
    FS,
    PostcardTx,
    PostcardRx,
    PostcardBuf,
    PostcardD,
> (
    filesystem: FS, 
    server: Server<PostcardTx, PostcardRx, PostcardBuf, PostcardD>,
)
where 
    FS: FileSystem,
    PostcardTx: WireTx + Clone,
    PostcardRx: WireRx,
    PostcardBuf: DerefMut<Target = [u8]>,
    PostcardD: Dispatch<Tx = PostcardTx>,
{
    let postcard_sender = server.sender();

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
