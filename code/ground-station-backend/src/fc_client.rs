//! Postcard-rpc client for the FC ↔ GS link (`fc-gs.sock`).
//!
//! Connects to the flight-computer-host as a postcard-rpc client, subscribes
//! to `RecordTopic` for telemetry, and writes records to storage.

use proto::transport::ipc::connect_client;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::routes::AppState;

/// Shared view of FC connection state, read by REST routes.
#[derive(Default)]
pub struct FcConnection {
    /// Human-readable error from the last disconnect, if any.
    pub last_error: Option<String>,
    /// Postcard-rpc client handle, cloned so routes can make endpoint calls.
    /// `Some` = connected, `None` = disconnected.
    pub client: Option<proto::PostcardClient>,
    /// Last measured FC ping latency. `None` when disconnected.
    pub latency: Option<std::time::Duration>,
}

impl FcConnection {
    /// Whether the FC is currently connected.
    pub fn connected(&self) -> bool {
        self.client.is_some()
    }

    /// Mark the connection as disconnected, clearing the client handle.
    pub(super) fn set_disconnected(&mut self, reason: String) {
        self.client = None;
        self.last_error = Some(reason);
    }
}

impl FcConnection {
    /// Measure FC ping latency and update the connection state.
    async fn update_ping(&mut self) {
        if let Some(client) = &self.client {
            let start = std::time::Instant::now();
            if let Ok(Ok(resp)) = tokio::time::timeout(
                Config::ENDPOINT_TIMEOUT,
                client.service::<proto::PingEndpoint>(&proto::PingRequest::from(Config::PING_PAYLOAD)),
            ).await
                && *resp == Config::PING_PAYLOAD
            {
                self.latency = Some(start.elapsed());
            }
        } else {
            self.latency = None;
        }
    }
}

/// Run the periodic FC ping loop: measure latency and broadcast status.
///
/// Spawned as a standalone task — runs at `Config::PING_POLL` interval until
/// the broadcast channel closes or the process exits.
pub async fn run_ping_loop(state: AppState) {
    let mut ticker = tokio::time::interval(Config::PING_POLL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        ticker.tick().await;
        state.conn.write().await.update_ping().await;
        state.broadcast_status().await;
    }
}

/// Run the FC client loop with automatic reconnection.
///
/// Connects to the FC host, subscribes to `RecordTopic` for telemetry,
/// forwards records to storage and WebSocket, and reconnects after
/// [`Config::RECONNECT_INTERVAL`] on any failure or disconnect.
///
/// This function never returns — it loops until the process exits.
pub async fn run_fc_client(state: AppState) {
    loop {
        if let Err(e) = try_run_fc_client(&state).await {
            error!(error = %e, "FC client session failed, reconnecting");
            state.conn.write().await.set_disconnected(format!("{e}"));
            state.broadcast_status().await;
            tokio::time::sleep(Config::RECONNECT_INTERVAL).await;
        }
    }
}

/// Single FC session: connect, subscribe, receive records until disconnect.
///
/// Returns an error describing why the session ended. The caller should
/// log the error, mark disconnected, wait, and retry.
async fn try_run_fc_client(state: &AppState) -> anyhow::Result<()> {
    info!("Connecting to FC on {}...", utils::constants::GS_SOCKET_NAME);
    let client = connect_fc().await?;
    info!("Connected to FC on {}", utils::constants::GS_SOCKET_NAME);

    // Register client handle for REST routes.
    {
        let mut c = state.conn.write().await;
        c.last_error = None;
        c.client = Some(client.clone());
    }

    let mut sub = client.subscribe::<proto::RecordTopic>().await
        .map_err(|e| anyhow::anyhow!("subscribe to RecordTopic failed: {e}"))?;

    info!("Subscribed to RecordTopic, waiting for telemetry...");
    state.broadcast_status().await;

    // Receive records until the subscription drops (FC disconnects).
    while let Some(record) = sub.recv().await {
        let mut store = state.storage.write().await;
        if let Some(ref mut s) = *store
            && let Err(e) = s.store_record(record.clone()) {
            warn!(error = %e, "Failed to write record to storage");
        }

        // Broadcast to WebSocket clients as JSON.
        if let Ok(json) = serde_json::to_string(&utils::status::WsMessage::Record(record.clone()))
    && let Err(e) = state.ws_sender.send(json) {
        debug!("Failed to broadcast record (no WS clients): {}", e);
    }
    }

    anyhow::bail!("FC disconnected (subscription closed)")
}

async fn connect_fc() -> anyhow::Result<proto::PostcardClient> {
    let client = connect_client::<{ Config::CLIENT_QUEUE_DEPTH }>(utils::constants::GS_SOCKET_NAME)
        .await
        .map_err(|e| anyhow::anyhow!("connect to {} failed: {e}", utils::constants::GS_SOCKET_NAME))?;
    Ok(client)
}
