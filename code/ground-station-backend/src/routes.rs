//! Rocket route handlers for the GS REST API.
//!
//! All routes are scoped under `/api`.

use std::io::BufRead;
use std::path::PathBuf;
use std::sync::Arc;

use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::fc_client::FcConnection;

/// Shared application state managed by Rocket.
pub struct AppState {
    pub conn: Arc<RwLock<FcConnection>>,
    pub storage: Arc<RwLock<Option<crate::storage::RecordStorage>>>,
    pub log_dir: PathBuf,
}

// ---- Response types ----

#[derive(Serialize)]
pub struct StatusResponse {
    pub connected: bool,
    pub session_start: String,
    pub record_count: u64,
}

#[derive(Serialize)]
pub struct PingSuccess {
    pub latency_ms: f64,
}

#[derive(Serialize)]
pub struct PingError {
    pub error: String,
}

// ---- Routes ----

/// `GET /api/status` — FC connection state and session summary.
#[rocket::get("/status")]
pub async fn status(state: &State<AppState>) -> Json<StatusResponse> {
    let conn = state.conn.read().await;
    let store = state.storage.read().await;
    let (session_start, record_count) = match &*store {
        Some(s) => (s.session_start().to_string(), s.count()),
        None => (String::new(), 0),
    };
    Json(StatusResponse {
        connected: conn.connected,
        session_start,
        record_count,
    })
}

/// `GET /api/records` — all telemetry records from the current session.
#[rocket::get("/records")]
pub async fn records(
    state: &State<AppState>,
) -> Result<Json<Vec<proto::record::Record>>, rocket::response::status::NotFound<&'static str>> {
    let store = state.storage.read().await;
    match &*store {
        Some(s) => Ok(Json(s.records().to_vec())),
        None => Err(rocket::response::status::NotFound("no active session")),
    }
}

/// `GET /api/records/latest` — most recent telemetry record.
#[rocket::get("/records/latest")]
pub async fn records_latest(
    state: &State<AppState>,
) -> Result<Json<proto::record::Record>, rocket::response::status::NotFound<&'static str>> {
    let store = state.storage.read().await;
    match &*store {
        Some(s) => match s.latest_record() {
            Some(r) => Ok(Json(r.clone())),
            None => Err(rocket::response::status::NotFound("no records yet")),
        },
        None => Err(rocket::response::status::NotFound("no active session")),
    }
}

/// `GET /api/logs` — recent GS-side log lines from the current session.
///
/// Returns the last 200 JSON log entries from the combined log file.
#[rocket::get("/logs?<lines>")]
pub async fn logs(
    state: &State<AppState>,
    lines: Option<usize>,
) -> Result<Json<Vec<serde_json::Value>>, rocket::response::status::NotFound<&'static str>> {
    let max_lines = lines.unwrap_or(200).min(2000);
    let log_file = state.log_dir.join("log.json");

    let file = match std::fs::File::open(&log_file) {
        Ok(f) => f,
        Err(_) => return Err(rocket::response::status::NotFound("log file not available")),
    };

    let reader = std::io::BufReader::new(file);
    // Collect up to max_lines from the end.
    let mut all_lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(l) => all_lines.push(l),
            Err(_) => continue, // skip malformed lines
        }
    }

    let tail: Vec<serde_json::Value> = all_lines
        .into_iter()
        .rev()
        .take(max_lines)
        .filter_map(|l| serde_json::from_str(&l).ok())
        .collect();

    Ok(Json(tail))
}

/// `POST /api/commands/ping` — ping the FC, echo-check, return round-trip latency.
///
/// Sends `0xdeadbeef` and verifies the FC echoes it back.
///
/// - **200** `{"latency_ms": 1.23}` on success.
/// - **503** `{"error": "..."}` on failure (disconnected, timeout, echo mismatch).
#[rocket::post("/commands/ping")]
pub async fn ping(
    state: &State<AppState>,
) -> Result<Json<PingSuccess>, Custom<Json<PingError>>> {
    let client = {
        let c = state.conn.read().await;
        c.client.clone()
    };

    let Some(client) = client else {
        return Err(Custom(
            Status::ServiceUnavailable,
            Json(PingError {
                error: "FC not connected".into(),
            }),
        ));
    };

    let payload: u32 = 0xdeadbeef;
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
                Err(Custom(
                    Status::InternalServerError,
                    Json(PingError {
                        error: format!(
                            "ping echo mismatch: sent {payload:#x}, got {:#x}",
                            *resp,
                        ),
                    }),
                ))
            } else {
                Ok(Json(PingSuccess {
                    latency_ms: latency.as_secs_f64() * 1000.0,
                }))
            }
        }
        Ok(Err(e)) => Err(Custom(
            Status::InternalServerError,
            Json(PingError {
                error: format!("ping failed: {e}"),
            }),
        )),
        Err(_) => Err(Custom(
            Status::RequestTimeout,
            Json(PingError {
                error: "ping timed out".into(),
            }),
        )),
    }
}
