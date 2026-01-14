use log::error;
use tokio::{sync::{broadcast, mpsc}};
use crate::{physics::{engine::PhysicsEngine, state::PhysicsState}, runtime::commands::{FlightComputerCommand, SimulatorCommand}, scripted_scenario::{scripted_arm_command, scripted_ignition_command}};
use crate::config::SimulatorConfig;

pub async fn simulator_loop(
    mut sim_rx: mpsc::Receiver<SimulatorCommand>,
    state_tx: broadcast::Sender<PhysicsState>,
) {
    let mut engine = PhysicsEngine::default();
    let config = SimulatorConfig::default();

    let mut physics_ticker = config.time_step_interval;
    let mut data_acquisition_ticker = config.data_acquisition_interval;

    loop {
        tokio::select! {
            _ = physics_ticker.tick() => {
                engine.step();
            },

            _ = data_acquisition_ticker.tick() => {
                if state_tx.send(engine.state()).is_err() { error!("Failed to broadcast PhysicsState"); }
            },

            Some(command) = sim_rx.recv() => {
                engine.handle_command(command);
            }
        }
    }
}

pub async fn scripted_scenario(
    sim_tx: mpsc::Sender<SimulatorCommand>,
    fc_tx: broadcast::Sender<FlightComputerCommand>,
) {
    scripted_ignition_command(sim_tx).await;
    scripted_arm_command(fc_tx).await;
}
