pub mod api;
pub mod physics;
pub mod runtime;
pub mod scripted_scenario;
pub mod config;

use tokio::sync::{broadcast, mpsc};

use crate::{config::SimulatorConfig, runtime::orchestrator};

pub fn start() -> api::ApiHandle {
    let (sim_tx, sim_rx) = mpsc::channel(SimulatorConfig::SIMULATOR_COMMAND_CAPACITY);
    let (fc_tx, fc_rx) = broadcast::channel(SimulatorConfig::FLIGHT_COMPUTER_COMMAND_CAPACITY);
    let (state_tx, state_rx) = broadcast::channel(SimulatorConfig::PHYSICS_STATE_CAPACITY);

    tokio::spawn(orchestrator::simulator_loop(
        sim_rx,
        state_tx,
    ));

    tokio::spawn(orchestrator::scripted_scenario(
        sim_tx.clone(), 
        fc_tx
    ));

    api::ApiHandle::new(sim_tx, fc_rx, state_rx)
}
