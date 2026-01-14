use tokio::{sync::{broadcast, mpsc}, time::sleep};
use crate::{config::SimulatorConfig, runtime::commands::{FlightComputerCommand, SimulatorCommand}};

pub async fn scripted_ignition_command(sim_tx: mpsc::Sender<SimulatorCommand>) {
    sleep(SimulatorConfig::ACTIVATION_DELAY_IGNITION).await;
    sim_tx.send(SimulatorCommand::Ignition).await.expect("Failed to send Ignition command");
}

pub async fn scripted_arm_command(fc_tx: broadcast::Sender<FlightComputerCommand>) {
    sleep(SimulatorConfig::ACTIVATION_DELAY_ARM).await;
    fc_tx.send(FlightComputerCommand::Arm).expect("Failed to send Arm command");
}
