use flight_computer_lib::{config::FlightComputerConfig, tasks::FlightComputer};

use crate::{board::{SimBoard, SimBoardConfig}, logging::{Logging, LoggingConfig}, simulator::SimulatorConfig};

mod board;
mod logging;
mod simulator;

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

    FlightComputer {
        config: config.flight_computer,
        altimeter,
        arm_button,
        deployment_system,
        gps,
        imu,
        sd_card,
        sd_card_detect,
        sd_card_status_led,
        postcard_sender,
    }.start().await;

    tracing::info!("Application tasks exited");
}
