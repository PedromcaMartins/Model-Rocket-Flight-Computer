use log::error;
use tokio::{sync::{broadcast, mpsc}};
use crate::{physics::{engine::PhysicsEngine, state::PhysicsState}, runtime::commands::{FlightComputerCommand, SimulatorCommand}, scripted_scenario::{scripted_arm_command, scripted_ignition_command}};
use crate::config::SimulatorConfig;

pub async fn scripted_scenario(
    sim_tx: mpsc::Sender<SimulatorCommand>,
    fc_tx: broadcast::Sender<FlightComputerCommand>,
) {
    scripted_ignition_command(sim_tx).await;
    scripted_arm_command(fc_tx).await;
}
