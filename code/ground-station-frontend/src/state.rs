use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use arc_swap::ArcSwap;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::backend::{BackendClient, Status, WsMessage};
use crate::history::RollingHistory;

/// Shared application state, accessible from both the WS reader task and the
/// TUI render loop. Concurrency model per spec §4.2.
pub struct AppState<B: BackendClient> {
    // -- Connection --
    pub status: Mutex<Status>,
    pub ping_heartbeat: AtomicBool,
    pub last_error: Mutex<Option<String>>,
    pub last_cmd_result: Mutex<Option<String>>,

    // -- Telemetry (latest values) --
    pub latest_record: ArcSwap<Option<proto::record::Record>>,

    // -- History (rolling windows) --
    pub altitude: Mutex<RollingHistory<proto::sensor_data::Altitude>>,
    pub gps_history: Mutex<RollingHistory<proto::sensor_data::GpsCoordinates>>,

    // -- Events --
    pub transitions: Mutex<Vec<(Instant, proto::flight_state::FlightState)>>,

    // -- Logs --
    pub log_buffer: Mutex<VecDeque<String>>,

    // -- Backend client --
    pub backend: Arc<B>,

    // -- Internal: timing baseline --
    pub session_start_time: Mutex<Option<Instant>>,
    /// Timestamp of the last record received — used for "Last seen: Xs ago" disconnect badge.
    pub last_record_time: Mutex<Option<Instant>>,

    // -- WS reader cancellation --
    pub reader_cancel: Mutex<CancellationToken>,
}

impl<B: BackendClient> AppState<B> {
    pub fn new(backend: Arc<B>) -> Self {
        Self {
            status: Mutex::new(Status {
                connected: false,
                session_start: chrono::DateTime::UNIX_EPOCH,
                record_count: 0,
                latency: None,
            }),
            ping_heartbeat: AtomicBool::new(false),
            last_error: Mutex::new(None),
            last_cmd_result: Mutex::new(None),
            latest_record: ArcSwap::new(Arc::new(None)),
            altitude: Mutex::new(RollingHistory::new(
                crate::config::Config::HISTORY_WINDOW,
            )),
            gps_history: Mutex::new(RollingHistory::new(
                crate::config::Config::HISTORY_WINDOW,
            )),
            transitions: Mutex::new(Vec::new()),
            log_buffer: Mutex::new(VecDeque::with_capacity(
                crate::config::Config::LOG_BUFFER_CAPACITY,
            )),
            backend,
            session_start_time: Mutex::new(None),
            last_record_time: Mutex::new(None),
            reader_cancel: Mutex::new(CancellationToken::new()),
        }
    }
}

/// Run the WS reader with automatic reconnection.
///
/// On disconnect or error, sleeps [`Config::RECONNECT_INTERVAL`] and retries.
/// Exits only when the cancellation token in [`AppState::reader_cancel`] is triggered (shutdown).
pub async fn run_ws_reader<B: BackendClient>(
    backend: Arc<B>,
    state: Arc<AppState<B>>,
) {
    let cancel = state.reader_cancel.lock().unwrap_or_else(|p| p.into_inner()).clone();
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            result = try_run_ws_reader(&backend, &state) => {
                if let Err(e) = result {
                    warn!("WS session failed: {e}");
                }
                tokio::time::sleep(crate::config::Config::RECONNECT_INTERVAL).await;
            }
        }
    }
}

/// Single WS session: connect, stream messages, exit on disconnect.
async fn try_run_ws_reader<B: BackendClient>(
    backend: &Arc<B>,
    state: &Arc<AppState<B>>,
) -> anyhow::Result<()> {
    let mut stream = backend.connect_ws().await?;
    state.status.lock().unwrap_or_else(|p| p.into_inner()).connected = true;

    while let Some(msg) = stream.next().await {
        match msg {
            WsMessage::Record(record) => {
                let now = Instant::now();
                *state.last_record_time.lock().unwrap_or_else(|p| p.into_inner()) = Some(now);

                let mut start = state.session_start_time.lock().unwrap_or_else(|p| p.into_inner());
                if start.is_none() {
                    *start = Some(now);
                }
                drop(start);

                state.latest_record.store(Arc::new(Some(record.clone())));
                state.status.lock().unwrap_or_else(|p| p.into_inner()).record_count += 1;

                match record.payload() {
                    proto::record::RecordData::FlightState(fs) => {
                        state.transitions.lock().unwrap_or_else(|p| p.into_inner()).push((now, *fs));
                    }
                    proto::record::RecordData::Altimeter(data) => {
                        state.altitude.lock().unwrap_or_else(|p| p.into_inner()).push(now, data.altitude);
                    }
                    proto::record::RecordData::Gps(data) => {
                        state.gps_history.lock().unwrap_or_else(|p| p.into_inner()).push(now, data.coordinates.clone());
                    }
                    _ => {}
                }
            }
            WsMessage::Status(status) => {
                let mut s = state.status.lock().unwrap_or_else(|p| p.into_inner());
                s.latency = status.latency;
                s.session_start = status.session_start;
                drop(s);
                state.ping_heartbeat.fetch_xor(true, std::sync::atomic::Ordering::Relaxed);
            }
            WsMessage::Log(_) => {}
        }
    }

    anyhow::bail!("WS connection closed")
}
