//! Rocket route handlers for the GS REST API + WebSocket endpoint.
//!
//! All routes are scoped under `/api`.

use std::sync::Arc;

use rocket::futures::SinkExt;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use tokio::sync::{broadcast, RwLock};
use tokio::sync::broadcast::error::RecvError;

use tracing::{debug, info, warn};

use crate::config::Config;
use crate::fc_client::FcConnection;
use crate::storage::RecordStorage;

/// Shared broadcast sender for WebSocket clients.
/// Pre-serialized JSON strings (one per record/status message).
pub type WsSender = broadcast::Sender<String>;

/// Shared application state managed by Rocket.
#[derive(Clone)]
pub struct AppState {
    pub conn: Arc<RwLock<FcConnection>>,
    pub storage: Arc<RwLock<Option<RecordStorage>>>,
    pub ws_sender: WsSender,
}

impl AppState {
    /// Broadcast the current FC connection status as a JSON string to WebSocket clients.
    pub(crate) async fn broadcast_status(&self) {
        let conn = self.conn.read().await;
        let store = self.storage.read().await;
        let (session_start, record_count) = match &*store {
            Some(s) => (s.session_start(), s.count()),
            None => (chrono::DateTime::UNIX_EPOCH, 0),
        };
        if let Ok(json) = serde_json::to_string(&utils::status::WsMessage::Status(
            utils::status::Status { connected: conn.connected(), session_start, record_count, latency: conn.latency },
        )) && let Err(e) = self.ws_sender.send(json) {
            debug!("Failed to send status update (no WS clients): {}", e);
        }
    }

    /// Extract the FC postcard client from shared state.
    ///
    /// Returns a 503 error response when the FC is disconnected.
    pub(crate) async fn get_fc_client(&self) -> Result<proto::PostcardClient, Custom<Json<CommandError>>> {
        match self.conn.read().await.client.clone() {
            Some(client) => Ok(client),
            None => Err(json_error(Status::ServiceUnavailable, "FC not connected")),
        }
    }
}

// ---- Response types ----

pub use utils::status::{CommandError, CommandSuccess, PingSuccess, Status as StatusResponse};

// ---- Helpers ----

fn json_error(status: Status, msg: impl Into<String>) -> Custom<Json<CommandError>> {
    Custom(status, Json(CommandError { error: msg.into() }))
}

// ---- Routes ----

/// `GET /api/status` — FC connection state and session summary.
#[rocket::get("/status")]
pub async fn status(state: &State<AppState>) -> Json<StatusResponse> {
    let conn = state.conn.read().await;
    let store = state.storage.read().await;
        let (session_start, record_count) = match &*store {
            Some(s) => (s.session_start(), s.count()),
            None => (chrono::DateTime::UNIX_EPOCH, 0),
        };
        Json(StatusResponse {
        connected: conn.connected(),
        session_start,
        record_count,
        latency: None,
    })
}

/// `GET /api/records` — telemetry records from the current session.
///
/// Supports optional `?limit=N` to return only the last N records.
///
/// Rank 1 (lower priority than `ws_events` at rank 0) so that
/// WebSocket upgrade requests hit the WS handler first.
#[rocket::get("/records?<limit>", rank = 1)]
pub async fn records(
    state: &State<AppState>,
    limit: Option<usize>,
) -> Result<Json<Vec<proto::record::Record>>, rocket::response::status::NotFound<&'static str>> {
    let store = state.storage.read().await;
    match &*store {
        Some(s) => {
            let all = s.records();
            let records = match limit {
                Some(n) => {
                    let start = all.len().saturating_sub(n);
                    all[start..].to_vec()
                }
                None => all.to_vec(),
            };
            Ok(Json(records))
        }
        None => Err(rocket::response::status::NotFound("no active session")),
    }
}

/// `POST /api/commands/ping` — ping the FC, echo-check, return round-trip latency.
///
/// Sends `Config::PING_PAYLOAD` and verifies the FC echoes it back.
///
/// - **200** `{"latency"}` on success.
/// - **503** `{"error": "..."}` on failure (disconnected, timeout, echo mismatch).
#[rocket::post("/commands/ping")]
pub async fn ping(
    state: &State<AppState>,
) -> Result<Json<PingSuccess>, Custom<Json<CommandError>>> {
    debug!("ping requested");
    let client = state.get_fc_client().await?;

    let payload = Config::PING_PAYLOAD;
    let start = std::time::Instant::now();

    match tokio::time::timeout(
        Config::ENDPOINT_TIMEOUT,
        client.service::<proto::PingEndpoint>(&proto::PingRequest::from(payload)),
    )
    .await
    {
        Ok(Ok(resp)) => {
            let latency = start.elapsed();
            if *resp != payload {
                warn!("ping echo mismatch: sent {payload:#x}, got {:#x}", *resp);
                Err(json_error(
                    Status::InternalServerError,
                    format!("ping echo mismatch: sent {payload:#x}, got {:#x}", *resp),
                ))
            } else {
                debug!("ping OK: {}ms", latency.as_millis());
                Ok(Json(PingSuccess { latency }))
            }
        }
        Ok(Err(e)) => {
            warn!("ping failed: {e}");
            Err(json_error(Status::InternalServerError, format!("ping failed: {e}")))
        }
        Err(_) => {
            warn!("ping timed out");
            Err(json_error(Status::RequestTimeout, "ping timed out"))
        }
    }
}

/// `GET /api/records` (WebSocket upgrade) — live telemetry stream.
///
/// On connect, subscribes to the broadcast channel and forwards all
/// record/status messages to the WS client. Regular GET requests fall
/// through to the JSON `records` handler.
///
/// Rank 0 (default) so this route is tried before the REST `records`
/// handler (rank 1). Rocket falls through to the REST handler when
/// the request lacks WebSocket upgrade headers.
#[rocket::get("/records")]
pub fn ws_events(
    ws: rocket_ws::WebSocket,
    state: &State<AppState>,
) -> rocket_ws::Channel<'static> {
    let mut rx = state.ws_sender.subscribe();
    ws.channel(move |mut stream| Box::pin(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if stream
                        .send(rocket_ws::Message::Text(msg))
                        .await
                        .is_err()
                    {
                        warn!("WS client disconnected (send error)");
                        break;
                    }
                }
                Err(RecvError::Closed) => {
                    info!("WS broadcast channel closed");
                    break;
                }
                Err(RecvError::Lagged(n)) => {
                    warn!("WS client lagged behind by {n} messages");
                    continue;
                }
            }
        }
        Ok(())
    }))
}

/// `POST /api/commands/arm` — arm the FC flight computer.
///
/// Placeholder for M3.2 — validates FC connectivity but the actual postcard-rpc
/// arm endpoint is wired in M3.3 (sim-gs.sock integration).
///
/// - **200** `{"status": "accepted"}` when FC is connected.
/// - **503** `{"error": "..."}` when FC is disconnected.
#[rocket::post("/commands/arm")]
pub async fn arm(
    state: &State<AppState>,
) -> Result<Json<CommandSuccess>, Custom<Json<CommandError>>> {
    let _client = state.get_fc_client().await?;
    info!("arm command accepted");
    Ok(Json(CommandSuccess {
        status: "accepted".into(),
    }))
}

/// `POST /api/commands/ignite` — ignite the rocket motor.
///
/// Placeholder for M3.2 — validates FC connectivity but the actual postcard-rpc
/// ignite endpoint is wired in M3.3 (sim-gs.sock integration).
///
/// - **200** `{"status": "accepted"}` when FC is connected.
/// - **503** `{"error": "..."}` when FC is disconnected.
#[rocket::post("/commands/ignite")]
pub async fn ignite(
    state: &State<AppState>,
) -> Result<Json<CommandSuccess>, Custom<Json<CommandError>>> {
    let _client = state.get_fc_client().await?;
    info!("ignite command accepted");
    Ok(Json(CommandSuccess {
        status: "accepted".into(),
    }))
}
