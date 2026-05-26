use anyhow::Context;
use proto::flight_state::FlightState;
use tokio::sync::{mpsc, watch};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{config::SimulatorConfig, flight_computer::FcCommand, types::ForceEvent};

pub async fn run_scripted(
    scripted_cmd_tx: mpsc::Sender<FcCommand>,
    physics_trigger_tx: mpsc::Sender<ForceEvent>,
    fc_state_rx: watch::Receiver<FlightState>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    // Arming FC
    info!(delay_ms = SimulatorConfig::ARM_DELAY.as_millis(), "scripted: waiting for arm delay");
    tokio::select! {
        _ = cancel.cancelled() => anyhow::bail!("cancelled"),
        _ = tokio::time::sleep(SimulatorConfig::ARM_DELAY) => {}
    }
    scripted_cmd_tx.send(FcCommand::Arm).await.context("scripted_cmd_tx receiver dropped")?;
    info!("scripted: arm sent, waiting for FC to arm");

    wait_for_armed(fc_state_rx, cancel.clone()).await?;

    // Igniting motors
    info!(delay_ms = SimulatorConfig::IGNITION_DELAY.as_millis(), "scripted: FC armed, waiting for ignition");
    tokio::select! {
        _ = cancel.cancelled() => anyhow::bail!("cancelled"),
        _ = tokio::time::sleep(SimulatorConfig::IGNITION_DELAY) => {}
    }
    physics_trigger_tx
        .send(ForceEvent::MotorThrust)
        .await
        .context("physics trigger receiver dropped")?;
    info!("scripted: ignition sent");
    Ok(())
}

async fn wait_for_armed(
    mut rx: watch::Receiver<FlightState>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    loop {
        if *rx.borrow() == FlightState::Armed {
            return Ok(());
        }
        tokio::select! {
            _ = cancel.cancelled() => anyhow::bail!("scripted: cancelled while waiting for arm"),
            r = rx.changed() => r.context("scripted: FC state watch closed while waiting for arm")?,
        }
    }
}
