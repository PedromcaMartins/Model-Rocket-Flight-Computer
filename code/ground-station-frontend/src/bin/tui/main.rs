//! Ground Station Frontend — ratatui TUI binary.
//!
//! Entry point: initialises tracing, bootstraps state, spawns WS reader
//! and heartbeat poller, then runs the blocking TUI event loop.

use std::sync::Arc;

use tracing::info;

use ground_station_frontend::backend::WsBackend;
use ground_station_frontend::state::{run_ws_reader, AppState};

mod controls;
mod logs;
mod render;
mod telemetry;
mod tui;

// ---------------------------------------------------------------------------
// ActiveTab enum
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Telemetry,
    Logs,
    Controls,
}

impl ActiveTab {
    fn next(self) -> Self {
        match self {
            Self::Telemetry => Self::Logs,
            Self::Logs => Self::Controls,
            Self::Controls => Self::Telemetry,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Telemetry => Self::Controls,
            Self::Logs => Self::Telemetry,
            Self::Controls => Self::Logs,
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::logging::install_panic_hook();

    let _guard = utils::logging::init_tracing(utils::logging::LogConfig {
        log_root: utils::workspace::workspace_root().join("logs/gs_frontend"),
        stdout_level: utils::constants::STDOUT_LOG_LEVEL,
        ui: utils::logging::UiConfig::Stdout,
    })?;

    info!("Starting Ground Station Frontend");

    let backend = Arc::new(WsBackend::new());
    let state = Arc::new(AppState::new(backend.clone()));

    // Spawn the auto-reconnecting WS reader.
    let reader_handle = tokio::spawn(run_ws_reader(backend.clone(), state.clone()));

    // Run the TUI event loop (blocks until user quits).
    tui::run_tui(&state)?;

    // Cleanup: cancel WS reader and wait for it to exit.
    state.reader_cancel.lock().unwrap_or_else(|p| p.into_inner()).cancel();
    let _ = reader_handle.await;

    info!("Ground Station Frontend shutting down");
    Ok(())
}



