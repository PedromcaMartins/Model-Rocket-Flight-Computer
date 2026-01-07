use core::ops::DerefMut;

use defmt_or_log::info;
use embassy_futures::{select::select6, select::Either6};
use postcard_rpc::server::{Dispatch, Sender, Server, WireRx, WireTx};

use crate::{interfaces::{FileSystem, impls::simulation::{altimeter::SimAltimeter, arm_button::SimButton, deployment_system::SimParachute, gps::SimGps, imu::SimImu, sd_card_led::SimSdCardLed}}, tasks::{altimeter_task, finite_state_machine_task, gps_task, imu_task, postcard_server_task, sd_card_task}};

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
    let altimeter_task = altimeter_task(
        SimAltimeter, 
        &postcard_sender,
    );

    let finite_state_machine_task = finite_state_machine_task(
        SimButton, 
        SimParachute::new(&postcard_sender), 
    );

    let gps_task = gps_task(
        SimGps,
        &postcard_sender,
    );

    let imu_task = imu_task(
        SimImu, 
        &postcard_sender,
    );

    let sd_card_task = sd_card_task(
        sd_card, 
        SimSdCardLed::new(&postcard_sender), 
    );

    let postcard_task = postcard_server_task(server);

    defmt_or_log::trace!("TICK_HZ: {:?}", embassy_time::TICK_HZ);

    #[allow(clippy::ignored_unit_patterns)]
    match select6(
        altimeter_task, 
        finite_state_machine_task, 
        gps_task, 
        imu_task,
        sd_card_task,
        postcard_task,
    ).await {
        Either6::First(_) => { info!("Altimeter task ended") },
        Either6::Second(_) => { info!("Finite State Machine task ended") },
        Either6::Third(_) => { info!("GPS task ended") },
        Either6::Fourth(_) => { info!("IMU task ended") },
        Either6::Fifth(_) => { info!("SD Card task ended") },
        Either6::Sixth(_) => { info!("Postcard Server task ended") },
    }

    info!("Flight Computer finished!");
}
