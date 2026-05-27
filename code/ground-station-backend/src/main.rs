//! Ground-station backend — REST API + telemetry storage for the FC link.
//!
//! Connects to the flight-computer-host on `fc-gs.sock`, subscribes to
//! `RecordTopic` for telemetry, stores records to NDJSON, and serves
//! a REST/JSON API for the frontend.
//!
//! ## Config (compile-time constants in [`config`])
//!
//! | Constant | Default | Purpose |
//! |---|---|---|
//! | `FC_SOCKET_PATH` | `"fc-gs.sock"` | Namespaced local-socket path |
//! | `REST_HOST` | `127.0.0.1` | REST server bind address |
//! | `REST_PORT` | `8000` | REST server port |
//! | `RECORDS_ROOT_DIR` | `logs/gs_records` | Session NDJSON output directory |
//! | `LOG_ROOT_DIR` | `logs/gs_backend` | Per-level JSON log directory |
//!
//! ## Logging
//!
//! Per-level JSON files in `logs/gs_backend/<timestamp>/` plus stdout filtered by
//! `RUST_LOG` (default `INFO`). Mirrors `flight-computer-host/src/logging.rs`.

mod config;
mod fc_client;
mod logging;
mod routes;
mod storage;

use std::sync::Arc;

use tokio::sync::RwLock;

use config::Config as GsConfig;
use fc_client::FcConnection;
use logging::LoggingGuard;
use routes::AppState;
use storage::RecordStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::install_panic_hook();
    let guard: LoggingGuard = logging::init_tracing()?;

    // Shared state between the FC client task and REST routes.
    let conn = Arc::new(RwLock::new(FcConnection::default()));
    let storage = Arc::new(RwLock::new(Some(RecordStorage::create()?)));
    let log_dir = guard.log_dir.clone();

    // Spawn the FC client loop (connects, subscribes, writes records).
    let fc_conn = conn.clone();
    let fc_storage = storage.clone();
    tokio::spawn(async move {
        fc_client::run_fc_client(fc_storage, fc_conn).await;
    });

    tracing::info!(
        "Starting REST API on {}:{}",
        GsConfig::REST_HOST,
        GsConfig::REST_PORT
    );

    let state = AppState { conn, storage, log_dir };

    let figment = rocket::figment::Figment::from(rocket::config::Config::default())
        .merge(("address", GsConfig::REST_HOST))
        .merge(("port", GsConfig::REST_PORT))
        .merge(("shutdown.ctrlc", GsConfig::CTRLC))
        .merge(("shutdown.grace", GsConfig::GRACE))
        .merge(("shutdown.merciless", GsConfig::MERCILESS));

    let _rocket = rocket::custom(figment)
        .manage(state)
        .mount(GsConfig::API_PATH, rocket::routes![
            routes::status,
            routes::records,
            routes::records_latest,
            routes::logs,
            routes::ping,
        ])
        .launch()
        .await?;

    Ok(())
}
