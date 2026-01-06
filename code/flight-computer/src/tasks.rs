use core::ops::DerefMut;

use defmt_or_log::info;
use embassy_futures::{select::select6, select::Either6};
use postcard_rpc::server::{Dispatch, Sender, Server, WireRx, WireTx};
use switch_hal::{InputSwitch, OutputSwitch, WaitSwitch};
use proto::{AltimeterMessage, GpsMessage, ImuMessage};

use crate::interfaces::{self, FileSystem, SensorDevice};

mod finite_state_machine;
pub use finite_state_machine::finite_state_machine_task;
mod imu;
pub use imu::imu_task;
mod altimeter;
pub use altimeter::altimeter_task;
mod gps;
pub use gps::gps_task;
mod sd_card;
pub use sd_card::sd_card_task;
pub mod postcard;
pub use postcard::postcard_server_task;

pub async fn start_flight_computer<
    Altimeter,
    ArmButton,
    DeploymentSystem,
    Gps,
    Imu,
    SdCard,
    SdCardDetect,
    SdCardStatusLed,
    PostcardTx,
    PostcardRx,
    PostcardBuf,
    PostcardD,
> (
    altimeter: Altimeter, 
    arm_button: ArmButton, 
    deployment_system: DeploymentSystem, 
    gps: Gps, 
    imu: Imu, 
    sd_card: SdCard, 
    sd_card_detect: SdCardDetect, 
    sd_card_status_led: SdCardStatusLed, 
    postcard_sender: Sender<PostcardTx>,
    server: Server<PostcardTx, PostcardRx, PostcardBuf, PostcardD>,
)
where 
    Altimeter: SensorDevice<DataMessage = AltimeterMessage>,
    ArmButton: WaitSwitch + 'static,
    <ArmButton as WaitSwitch>::Error: core::fmt::Debug,
    DeploymentSystem: interfaces::DeploymentSystem,
    Gps: SensorDevice<DataMessage = GpsMessage>,
    Imu: SensorDevice<DataMessage = ImuMessage>,
    SdCard: FileSystem,
    SdCardDetect: InputSwitch,
    SdCardStatusLed: OutputSwitch,
    PostcardTx: WireTx + Clone,
    PostcardRx: WireRx,
    PostcardBuf: DerefMut<Target = [u8]>,
    PostcardD: Dispatch<Tx = PostcardTx>,
{
    let altimeter_task = altimeter_task(
        altimeter, 
        &postcard_sender,
    );

    let finite_state_machine_task = finite_state_machine_task(
        arm_button, 
        deployment_system, 
    );

    let gps_task = gps_task(
        gps,
        &postcard_sender,
    );

    let imu_task = imu_task(
        imu, 
        &postcard_sender,
    );

    let sd_card_task = sd_card_task(
        sd_card, 
        sd_card_detect, 
        sd_card_status_led, 
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
