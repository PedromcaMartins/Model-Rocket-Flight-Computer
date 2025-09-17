use flight_computer_lib::{config::{ApogeeDetectorConfig, TouchdownDetectorConfig}, embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal, watch::Watch}, tasks::{altimeter_task, finite_state_machine_task, gps_task, imu_task, sd_card_task}};
use telemetry_messages::{AltimeterMessage, Altitude, FlightState, GpsMessage, ImuMessage};
use tokio::join;

use crate::{board::{SimBoard, SimBoardConfig}, logging::{Logging, LoggingConfig}, simulator::SimulatorConfig};

mod board;
mod logging;
mod simulator;

static LATEST_ALTITUDE_SIGNAL: Signal<CriticalSectionRawMutex, Altitude> = Signal::new();

const FLIGHT_STATE_WATCH_CONSUMERS: usize = 2;
static FLIGHT_STATE_WATCH: Watch<CriticalSectionRawMutex, FlightState, FLIGHT_STATE_WATCH_CONSUMERS> = Watch::new();

const ALTIMETER_SD_CARD_CHANNEL_DEPTH: usize = 10;
const GPS_SD_CARD_CHANNEL_DEPTH: usize = 10;
const IMU_SD_CARD_CHANNEL_DEPTH: usize = 10;

static ALTIMETER_SD_CARD_CHANNEL: Channel<CriticalSectionRawMutex, AltimeterMessage, ALTIMETER_SD_CARD_CHANNEL_DEPTH> = Channel::new();
static GPS_SD_CARD_CHANNEL:       Channel<CriticalSectionRawMutex, GpsMessage, GPS_SD_CARD_CHANNEL_DEPTH> = Channel::new();
static IMU_SD_CARD_CHANNEL:       Channel<CriticalSectionRawMutex, ImuMessage, IMU_SD_CARD_CHANNEL_DEPTH> = Channel::new();

#[derive(Default)]
pub struct HostFlightComputerConfig {
    logging: LoggingConfig,
    sim_board: SimBoardConfig,
    simulator: SimulatorConfig,
    apogee_detector: ApogeeDetectorConfig,
    touchdown_detector: TouchdownDetectorConfig,
}

#[tokio::main]
async fn main() {
    let config = HostFlightComputerConfig::default();

    Logging::init(config.logging).await;

    let SimBoard {
        simulator,
        arm_button,
        deployment_system,
        altimeter,
        gps,
        imu,
        postcard_sender,
        postcard_host_client: _, // TODO: use ground-station-backend :D
        sd_card,
        sd_card_detect,
        sd_card_status_led,
    } = SimBoard::init(config.sim_board, config.simulator).await;

    simulator.start();

    let altimeter_task = tokio::spawn(altimeter_task(
        altimeter, 
        &LATEST_ALTITUDE_SIGNAL, 
        ALTIMETER_SD_CARD_CHANNEL.sender(), 
        postcard_sender.clone(),
    ));

    let finite_state_machine_task = tokio::spawn(finite_state_machine_task(
        arm_button, 
        deployment_system, 
        &LATEST_ALTITUDE_SIGNAL, 
        config.apogee_detector, 
        config.touchdown_detector, 
        FLIGHT_STATE_WATCH.sender(),
    ));

    let gps_task = tokio::spawn(gps_task(
        gps,
        GPS_SD_CARD_CHANNEL.sender(), 
        postcard_sender.clone()
    ));

    let imu_task = tokio::spawn(imu_task(
        imu, 
        IMU_SD_CARD_CHANNEL.sender(), 
        postcard_sender.clone()
    ));

    let sd_card_task = tokio::spawn(sd_card_task(
        sd_card, 
        sd_card_detect, 
        sd_card_status_led, 
        ALTIMETER_SD_CARD_CHANNEL.receiver(), 
        GPS_SD_CARD_CHANNEL.receiver(), 
        IMU_SD_CARD_CHANNEL.receiver()
    ));

    tracing::info!("Application started");

    let res = join!(
        altimeter_task,
        finite_state_machine_task,
        gps_task,
        imu_task,
        sd_card_task,
    );

    tracing::info!("Application tasks exited with task results: {res:?}")
}
