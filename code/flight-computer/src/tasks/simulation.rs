use core::ops::DerefMut;

use defmt_or_log::info;
use embassy_futures::select::{Either, Either6, select, select6};
use postcard_rpc::server::{Dispatch, Sender, Server, WireRx, WireTx};
use proto::SimFileSystemLedTopic;

use crate::{interfaces::{FileSystem, impls::simulation::{altimeter::SimAltimeter, arming_system::SimArming, deployment_system::SimRecovery, led::SimLed, gps::SimGps, imu::SimImu}}, tasks::{finite_state_machine_task, groundstation_task, postcard_server_task, sensor_task, storage_task}};

pub async fn start_software_flight_computer<
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
    let postcard_task = postcard_server_task(server);
    let altimeter_task = sensor_task(SimAltimeter);
    let gps_task = sensor_task(SimGps);
    let imu_task = sensor_task(SimImu);

    let finite_state_machine_task = finite_state_machine_task(
        SimArming, 
        SimRecovery::new(&postcard_sender), 
    );

    let storage_task = storage_task(
        sd_card, 
        SimLed::<_, SimFileSystemLedTopic>::new(&postcard_sender), 
    );

    let groundstation_task = groundstation_task(&postcard_sender);

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
