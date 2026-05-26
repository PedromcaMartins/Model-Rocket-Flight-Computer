pub mod engine;
pub mod state;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::config::SimulatorConfig;
use crate::physics::{engine::PhysicsEngine, state::PhysicsState};
use crate::types::ForceEvent;

/// Drives the physics engine: steps at `physics_time_step`, publishes
/// `PhysicsState` snapshot every `data_acquisition_step` steps, and applies
/// inbound forces. Exits when `cancel` fires.
pub async fn run_physics_loop(
    mut engine: PhysicsEngine,
    physics_state_tx: tokio::sync::watch::Sender<PhysicsState>,
    mut physics_force_rx: mpsc::Receiver<ForceEvent>,
    cancel: CancellationToken,
) {
    let mut physics_time_step_ticker = tokio::time::interval(SimulatorConfig::PHYSICS_TIME_STEP_INTERVAL);
    let mut fc_updater_ticker = tokio::time::interval(SimulatorConfig::DATA_ACQUISITION_INTERVAL);

    info!(
        physics_step_ms = SimulatorConfig::PHYSICS_TIME_STEP_INTERVAL.as_millis(),
        data_acquisition_ms = SimulatorConfig::DATA_ACQUISITION_INTERVAL.as_millis(),
        "physics loop starting"
    );

    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => break,
            _ = physics_time_step_ticker.tick() => {
                // force new forces / events
                while let Ok(force) = physics_force_rx.try_recv() {
                    engine.handle_force_event(force);
                }
                engine.step();
            },
            _ = fc_updater_ticker.tick() => {
                // update simulator sensors
                if let Err(e) = physics_state_tx.send(engine.state()) {
                    tracing::warn!("failed to send physics state: {}", e);
                }
            }
        }
    }
    info!("physics loop stopped");
}
