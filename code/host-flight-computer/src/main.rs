use flight_computer_lib::{config::FlightComputerConfig, tasks::{altimeter_task, finite_state_machine_task, gps_task, imu_task, sd_card_task, ALTIMETER_SD_CARD_CHANNEL, FLIGHT_STATE_WATCH, GPS_SD_CARD_CHANNEL, IMU_SD_CARD_CHANNEL, LATEST_ALTITUDE_SIGNAL}};
use tokio::select;

use crate::{board::{SimBoard, SimBoardConfig}, logging::{Logging, LoggingConfig}, simulator::SimulatorConfig};

mod board;
mod logging;
mod simulator;
mod sim_devices;

#[derive(Default)]
pub struct HostFlightComputerConfig {
    logging: LoggingConfig,
    sim_board: SimBoardConfig,
    simulator: SimulatorConfig,
    flight_computer: FlightComputerConfig,
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

    tracing::info!("Application started");

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
        config.flight_computer.apogee_detector, 
        config.flight_computer.touchdown_detector, 
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

    select! {
        _ = altimeter_task => tracing::info!("Altimeter task exited"),
        _ = finite_state_machine_task => tracing::info!("Finite State Machine task exited"),
        _ = gps_task => tracing::info!("GPS task exited"),
        _ = imu_task => tracing::info!("IMU task exited"),
        _ = sd_card_task => tracing::info!("SD Card task exited"),
        _ = tokio::signal::ctrl_c() => tracing::info!("Received Ctrl-C, shutting down"),
    }
}
