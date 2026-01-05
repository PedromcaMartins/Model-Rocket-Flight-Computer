use core::ops::DerefMut;

use defmt_or_log::info;
use embassy_futures::{select::select6, select::Either6};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal, watch::Watch};
use postcard_rpc::server::{Dispatch, Sender, Server, WireRx, WireTx};
use switch_hal::{InputSwitch, OutputSwitch, WaitSwitch};
use proto::{AltimeterMessage, Altitude, FlightState, GpsMessage, ImuMessage};

use crate::{config::{FlightComputerConfig, TasksConfig}, interfaces::{self, FileSystem, SensorDevice}, tasks::postcard::postcard_server_task};


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


pub static LATEST_ALTITUDE_SIGNAL: Signal<CriticalSectionRawMutex, Altitude> = Signal::new();

pub static FLIGHT_STATE_WATCH: Watch<CriticalSectionRawMutex, FlightState, { TasksConfig::FLIGHT_STATE_WATCH_CONSUMERS }> = Watch::new();

pub static ALTIMETER_SD_CARD_CHANNEL: Channel<CriticalSectionRawMutex, AltimeterMessage, { TasksConfig::ALTIMETER_SD_CARD_CHANNEL_DEPTH }> = Channel::new();
pub static GPS_SD_CARD_CHANNEL:       Channel<CriticalSectionRawMutex, GpsMessage, { TasksConfig::GPS_SD_CARD_CHANNEL_DEPTH }> = Channel::new();
pub static IMU_SD_CARD_CHANNEL:       Channel<CriticalSectionRawMutex, ImuMessage, { TasksConfig::IMU_SD_CARD_CHANNEL_DEPTH }> = Channel::new();

pub struct FlightComputer<
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
>
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
    pub config: FlightComputerConfig, 
    pub altimeter: Altimeter, 
    pub arm_button: ArmButton, 
    pub deployment_system: DeploymentSystem, 
    pub gps: Gps, 
    pub imu: Imu, 
    pub sd_card: SdCard, 
    pub sd_card_detect: SdCardDetect, 
    pub sd_card_status_led: SdCardStatusLed, 
    pub postcard_sender: Sender<PostcardTx>,
    pub server: Server<PostcardTx, PostcardRx, PostcardBuf, PostcardD>,
}

impl<
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
> FlightComputer<
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
>
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
    pub async fn start(self) {
        let altimeter_task = altimeter_task(
            self.altimeter, 
            self.config.data_acquisition,
            &LATEST_ALTITUDE_SIGNAL, 
            ALTIMETER_SD_CARD_CHANNEL.sender(), 
            self.postcard_sender.clone(),
        );

        let finite_state_machine_task = finite_state_machine_task(
            self.arm_button, 
            self.deployment_system, 
            &LATEST_ALTITUDE_SIGNAL, 
            self.config.apogee_detector, 
            self.config.touchdown_detector, 
            FLIGHT_STATE_WATCH.sender(),
        );

        let gps_task = gps_task(
            self.gps,
            self.config.data_acquisition,
            GPS_SD_CARD_CHANNEL.sender(), 
            self.postcard_sender.clone()
        );

        let imu_task = imu_task(
            self.imu, 
            self.config.data_acquisition,
            IMU_SD_CARD_CHANNEL.sender(), 
            self.postcard_sender.clone()
        );

        let sd_card_task = sd_card_task(
            self.sd_card, 
            self.sd_card_detect, 
            self.sd_card_status_led, 
            self.config.log_filesystem,
            ALTIMETER_SD_CARD_CHANNEL.receiver(), 
            GPS_SD_CARD_CHANNEL.receiver(), 
            IMU_SD_CARD_CHANNEL.receiver()
        );

        let postcard_task = postcard_server_task(self.server);

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
}
