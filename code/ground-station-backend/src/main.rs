//! Ground-station backend — REST API + telemetry storage for the FC link.
//!
//! Connects to the flight-computer-host on `fc-gs.sock`, subscribes to
//! `RecordTopic` for telemetry, stores records to NDJSON, and serves
//! a REST/JSON API for the frontend.
//!
//! ## Config (shared constants in `utils::constants`)
//!
//! | Constant | Default | Purpose |
//! |---|---|---|
//! | `GS_HOST` | `"127.0.0.1"` | REST server bind address |
//! | `GS_PORT` | `8000` | REST server port |
//! | `RECORDS_ROOT_DIR` | `logs/gs_records` | Session NDJSON output directory |
//!
//! ## Logging
//!
//! Per-level JSON files in `<workspace_root>/logs/gs_backend/<timestamp>/` plus
//! stdout filtered by `RUST_LOG` (default `INFO`). Uses the shared `utils`
//! logging crate — see [`utils::logging`].

mod config;
mod fc_client;
mod routes;
mod storage;

use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};
use utils::logging::{LogConfig, LoggingGuard, UiConfig};
use utils::constants as c;
use utils::workspace;

use config::Config as GsConfig;
use fc_client::FcConnection;
use routes::AppState;
use storage::RecordStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::logging::install_panic_hook();
    let _guard: LoggingGuard = utils::logging::init_tracing(LogConfig {
        log_root: workspace::workspace_root().join("logs/gs_backend"),
        stdout_level: GsConfig::STDOUT_LOG_LEVEL,
        ui: UiConfig::Stdout,
    })?;

    // Shared state between the FC client task and REST routes.
    let conn = Arc::new(RwLock::new(FcConnection::default()));
    let storage = Arc::new(RwLock::new(Some(RecordStorage::create()?)));

    // Broadcast channel for WS clients — FC records are forwarded here.
    let (ws_tx, _) = broadcast::channel(256);

    let state = AppState { conn, storage, ws_sender: ws_tx };

    // Spawn the FC client loop (connects, subscribes, writes records, broadcasts).
    tokio::spawn(fc_client::run_fc_client(state.clone()));

    // Spawn a periodic FC ping task — latency flows through WS status messages.
    tokio::spawn(fc_client::run_ping_loop(state.clone()));

    tracing::info!(
        "Starting REST API on {}:{}",
        c::GS_HOST,
        c::GS_PORT
    );

    let figment = rocket::figment::Figment::from(rocket::config::Config::default())
        .merge(("address", c::GS_HOST))
        .merge(("port", c::GS_PORT))
        .merge(("shutdown.ctrlc", GsConfig::CTRLC))
        .merge(("shutdown.grace", GsConfig::GRACE))
        .merge(("shutdown.merciless", GsConfig::MERCILESS));

    let _rocket = rocket::custom(figment)
        .manage(state)
        .mount(c::API_PATH, rocket::routes![
            routes::status,
            routes::records,
            routes::ping,
            routes::ws_events,
            routes::arm,
            routes::ignite,
        ])
        .launch()
        .await?;

    Ok(())
}
