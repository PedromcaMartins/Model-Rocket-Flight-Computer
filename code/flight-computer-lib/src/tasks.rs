use defmt_or_log::info;
use embassy_futures::{select::select5, select::Either5};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal, watch::Watch};
use postcard_rpc::server::{Sender, WireTx};
use switch_hal::{InputSwitch, OutputSwitch, WaitSwitch};
use telemetry_messages::{AltimeterMessage, Altitude, FlightState, GpsMessage, ImuMessage};

use crate::{config::{FlightComputerConfig, TasksConfig}, interfaces::{self, FileSystem, SensorDevice}};


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


static LATEST_ALTITUDE_SIGNAL: Signal<CriticalSectionRawMutex, Altitude> = Signal::new();

static FLIGHT_STATE_WATCH: Watch<CriticalSectionRawMutex, FlightState, { TasksConfig::FLIGHT_STATE_WATCH_CONSUMERS }> = Watch::new();

static ALTIMETER_SD_CARD_CHANNEL: Channel<CriticalSectionRawMutex, AltimeterMessage, { TasksConfig::ALTIMETER_SD_CARD_CHANNEL_DEPTH }> = Channel::new();
static GPS_SD_CARD_CHANNEL:       Channel<CriticalSectionRawMutex, GpsMessage, { TasksConfig::GPS_SD_CARD_CHANNEL_DEPTH }> = Channel::new();
static IMU_SD_CARD_CHANNEL:       Channel<CriticalSectionRawMutex, ImuMessage, { TasksConfig::IMU_SD_CARD_CHANNEL_DEPTH }> = Channel::new();

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
    pub postcard_sender: Sender<PostcardTx>
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
{
    pub async fn start(self) {
        let altimeter_task = altimeter_task(
            self.altimeter, 
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
            GPS_SD_CARD_CHANNEL.sender(), 
            self.postcard_sender.clone()
        );

        let imu_task = imu_task(
            self.imu, 
            IMU_SD_CARD_CHANNEL.sender(), 
            self.postcard_sender.clone()
        );

        let sd_card_task = sd_card_task(
            self.sd_card, 
            self.sd_card_detect, 
            self.sd_card_status_led, 
            ALTIMETER_SD_CARD_CHANNEL.receiver(), 
            GPS_SD_CARD_CHANNEL.receiver(), 
            IMU_SD_CARD_CHANNEL.receiver()
        );

        #[allow(clippy::ignored_unit_patterns)]
        match select5(
            altimeter_task, 
            finite_state_machine_task, 
            gps_task, 
            imu_task,
            sd_card_task,
        ).await {
            Either5::First(_) => { info!("Altimeter task ended") },
            Either5::Second(_) => { info!("Finite State Machine task ended") },
            Either5::Third(_) => { info!("GPS task ended") },
            Either5::Fourth(_) => { info!("IMU task ended") },
            Either5::Fifth(_) => { info!("SD Card task ended") },
        }

        info!("Flight Computer finished!");
    }
}
