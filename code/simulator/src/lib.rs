//! Physics simulator for the flight computer — drives the FC over postcard-rpc
//! by publishing sensor data and reacting to actuator commands.
//!
//! This library is shared between two binaries (`host` and `pil`) that differ
//! only in transport (interprocess socket vs USB). It provides:
//!
//! - **Physics engine** (`physics/`) — rocket flight model (1D parabolic MVP,
//!   extensible to 3D kinematic attitude).
//! - **Scripted scenario** (`scripted/`) — compile-time event sequences that
//!   drive the FC through pre-defined flight phases.
//! - **Postcard-rpc client** (`flight_computer/`) — sends sensor data,
//!   receives FC actuator commands.
//! - **TUI** (`tui/`) — read-only ratatui dashboard for live telemetry.
//! - **Structured logging** — delegates to the shared `utils::logging` crate for
//!   per-level JSON files and combined `log.json`; the TUI log panel reads
//!   from `utils::logging::LOG_BUFFER`.
//!
//! See [`README.md`](README.md) for the crate overview and
//! [`spec.md`](spec.md) for the detailed design.

pub mod config;
pub mod connect;
pub mod flight_computer;
pub mod physics;
pub mod scripted;
pub mod tui;
pub mod types;

use std::sync::Arc;

use arc_swap::ArcSwap;
use proto::{PostcardClient, flight_state::FlightState};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::info;

use config::Config;
use flight_computer::{FcCommand, run_fc_client};
use physics::{engine::PhysicsEngine, run_physics_loop, state::PhysicsState};
use scripted::run_scripted;
use tui::run_tui;
use types::{ForceEvent, SimActuatorSnapshot};

pub async fn run_simulator(
    client: PostcardClient,
    cancel: CancellationToken,
    tui_cancel: CancellationToken,
) -> anyhow::Result<()> {
    info!("simulator starting");

    // PhysicsState: physics_engine -> fc_client + TUI
    // Published at DATA_ACQUISITION_INTERVAL, contains position/velocity/acceleration/attitude
    let (physics_to_fc, to_fc) = tokio::sync::watch::channel(PhysicsState::default());
    let to_tui = to_fc.clone();

    // FcCommand: scripted -> fc_client
    // Scripted scenario sends Arm after ARM_DELAY, then waits for armed confirmation
    let (script_to_fc, to_fc_client) = mpsc::channel::<FcCommand>(Config::FC_COMMAND_DEPTH);

    // ForceEvent: fc_client + scripted -> physics_engine
    // FC subscriber injects sensed forces from the real FC; scripted injects thrust on ignition
    let (fc_to_physics, to_physics) = mpsc::channel::<ForceEvent>(Config::FORCE_EVENT_DEPTH);
    let script_to_physics = fc_to_physics.clone();

    // FlightState: fc_client -> scripted
    // FC publishes its flight mode so scripted sequences events (wait-for-armed, etc.)
    let (fc_to_script, to_script) = tokio::sync::watch::channel(FlightState::default());

    // Actuator snapshot: FC subscriber (writer) -> TUI (reader)
    // ArcSwap<SimActuatorSnapshot>: FC writes servo/actuator status, TUI polls for display
    let actuator_snapshot = Arc::new(ArcSwap::new(SimActuatorSnapshot::default().into()));

    // PostcardClient: shared by FC subscriber + publisher tasks
    // Serial/network client for communicating with the real flight computer
    let sim_client = Arc::new(client);

    // ActiveForceEvent: registry of currently-active forces
    // Physics engine reads each tick; FC subscriber and scripted register events
    let active_events = Arc::default();

    // TUI reads the same force event registry
    let tui_events = Arc::clone(&active_events);

    let mut physics_handle = tokio::spawn({
        let physics_to_fc = physics_to_fc.clone();
        let cancel = cancel.clone();
        async move {
            run_physics_loop(
                PhysicsEngine::new(active_events),
                physics_to_fc,
                to_physics,
                cancel,
            )
            .await;
        }
    });

    // FC subscriber gets write access to the actuator snapshot
    let act_snapshot_clone = actuator_snapshot.clone();
    let mut fc_client_handle = tokio::spawn({
        let cancel = cancel.clone();
        async move {
            run_fc_client(
                sim_client,
                to_fc,
                to_fc_client,
                fc_to_physics,
                fc_to_script,
                act_snapshot_clone,
                cancel,
            )
            .await
        }
    });

    let scripted_handle = tokio::spawn({
        let cancel = cancel.clone();
        async move {
            let _ = run_scripted(script_to_fc, script_to_physics, to_script, cancel).await;
        }
    });

    // TUI gets read access to the actuator snapshot and force events
    let tui_handle = tokio::spawn({
        let active_events = Arc::clone(&tui_events);
        let cancel = cancel.clone();
        let tui_cancel = tui_cancel.clone();
        let act_snapshot = actuator_snapshot.clone();
        async move {
            run_tui(to_tui, active_events, act_snapshot, cancel, tui_cancel).await
        }
    });

    tokio::select! {
        biased;
        () = tui_cancel.cancelled() => {
            // User pressed q in TUI (or Ctrl-C), cascade shutdown
            cancel.cancel();
        }
        () = cancel.cancelled() => {
            info!("shutdown signal received, winding down");
        }
        result = &mut fc_client_handle => {
            if !cancel.is_cancelled() {
                cancel.cancel();
                match result {
                    Ok(inner) => {
                        if let Err(e) = inner {
                            tracing::warn!("FC disconnected: {e}");
                        } else {
                            tracing::warn!("FC disconnected (clean exit)");
                        }
                        tracing::warn!("simulator in degraded mode — press q to quit");
                    }
                    Err(join_err) => {
                        tracing::error!("fc_client task panicked: {join_err}");
                    }
                }
            }
        }
        _ = &mut physics_handle => {
            if !cancel.is_cancelled() {
                cancel.cancel();
                tracing::warn!("physics loop exited unexpectedly");
            }
        }
    }

    cancel.cancel();
    // Don't .await handles consumed by the select above — JoinHandle panics
    // on double-poll. abort() is safe on already-completed handles.
    fc_client_handle.abort();
    physics_handle.abort();
    let _ = scripted_handle.await;
    match tui_handle.await {
        Ok(Err(e)) => tracing::error!("TUI error: {e}"),
        Err(join_err) => tracing::error!("TUI task panicked: {join_err}"),
        _ => {}
    }

    info!("simulator stopped");
    Ok(())
}
