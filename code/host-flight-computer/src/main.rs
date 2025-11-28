use flight_computer_lib::{config::FlightComputerConfig, tasks::{ALTIMETER_SD_CARD_CHANNEL, FLIGHT_STATE_WATCH, GPS_SD_CARD_CHANNEL, IMU_SD_CARD_CHANNEL, LATEST_ALTITUDE_SIGNAL, altimeter_task, finite_state_machine_task, gps_task, imu_task, postcard::postcard_server_task, sd_card_task}};
use tokio::select;

use crate::{board::{SimBoard, SimBoardConfig}, logging::{Logging, LoggingConfig}, simulator::SimulatorConfig};

mod board;
mod logging;
mod simulator;
mod sim_devices;
mod simulator_ui;

#[derive(Default)]
pub struct HostFlightComputerConfig {
    logging: LoggingConfig,
    sim_board: SimBoardConfig,
    simulator: SimulatorConfig,
    flight_computer: FlightComputerConfig,
    ground_station_backend_api: ground_station_backend::ApiConfig,
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
        postcard_server,
        postcard_client,
        sd_card,
        sd_card_detect,
        sd_card_status_led,
        ui,
    } = SimBoard::init(config.sim_board, config.simulator).await;

    tracing::info!("TICK_HZ: {:?}", embassy_time::TICK_HZ);

    let simulator_task = simulator.run();

    let altimeter_task = tokio::spawn(altimeter_task(
        altimeter, 
        config.flight_computer.data_acquisition,
        &LATEST_ALTITUDE_SIGNAL, 
        ALTIMETER_SD_CARD_CHANNEL.sender(), 
        postcard_server.sender(),
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
        config.flight_computer.data_acquisition,
        GPS_SD_CARD_CHANNEL.sender(), 
        postcard_server.sender()
    ));

    let imu_task = tokio::spawn(imu_task(
        imu, 
        config.flight_computer.data_acquisition,
        IMU_SD_CARD_CHANNEL.sender(), 
        postcard_server.sender()
    ));

    let sd_card_task = tokio::spawn(sd_card_task(
        sd_card, 
        sd_card_detect, 
        sd_card_status_led, 
        config.flight_computer.log_filesystem,
        ALTIMETER_SD_CARD_CHANNEL.receiver(), 
        GPS_SD_CARD_CHANNEL.receiver(), 
        IMU_SD_CARD_CHANNEL.receiver()
    ));

    let postcard_server_task = tokio::spawn(postcard_server_task(postcard_server));

    let ground_station_backend_task = tokio::spawn(
        ground_station_backend::start_api(
            postcard_client,
            config.ground_station_backend_api
        )
    );

    tracing::info!("Application started");

    select! {
        _ = simulator_task => tracing::info!("Simulator task exited"),
        _ = altimeter_task => tracing::info!("Altimeter task exited"),
        _ = finite_state_machine_task => tracing::info!("Finite State Machine task exited"),
        _ = gps_task => tracing::info!("GPS task exited"),
        _ = imu_task => tracing::info!("IMU task exited"),
        _ = sd_card_task => tracing::info!("SD Card task exited"),
        _ = postcard_server_task => tracing::info!("Postcard Server task exited"),
        _ = tokio::signal::ctrl_c() => tracing::info!("Received Ctrl-C, shutting down"),
        res = ground_station_backend_task => match res {
            Ok(_) => tracing::info!("Ground Station Backend task exited"),
            Err(e) => tracing::error!("Ground Station Backend task exited with error: {:?}", e),
        },
    }

    // TODO: simulator ui (state + actuators)
    let _ = ui;
}
