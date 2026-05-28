//! Postcard-rpc client for the FC ↔ GS link (`fc-gs.sock`).
//!
//! Connects to the flight-computer-host as a postcard-rpc client, subscribes
//! to `RecordTopic` for telemetry, and writes records to storage.

use std::sync::Arc;

use proto::transport::ipc::connect_client;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::storage::RecordStorage;

/// Shared view of FC connection state, read by REST routes.
#[derive(Default)]
pub struct FcConnection {
    /// Whether the FC is currently connected.
    pub connected: bool,
    /// Human-readable error from the last disconnect, if any.
    pub last_error: Option<String>,
    /// Postcard-rpc client handle, cloned so routes can make endpoint calls.
    pub client: Option<proto::PostcardClient>,
}

impl FcConnection {
    /// Mark the connection as disconnected, clearing the client handle.
    pub(super) fn set_disconnected(&mut self, reason: String) {
        self.connected = false;
        self.client = None;
        self.last_error = Some(reason);
    }
}

/// Run the FC client loop: connect, subscribe to records, forward to storage,
/// then mark disconnected on any error.
///
/// This function returns when the connection is lost or closed. The caller
/// does **not** retry — a disconnect is permanent for the session
/// (per M3.1 design: operator restarts GS to reconnect).
pub async fn run_fc_client(
    storage: Arc<RwLock<Option<RecordStorage>>>,
    conn: Arc<RwLock<FcConnection>>,
) {
    info!("Connecting to FC on {}...", utils::constants::GS_SOCKET_NAME);

    let client = match connect_fc().await {
        Ok(c) => {
            info!("Connected to FC on {}", utils::constants::GS_SOCKET_NAME);
            // Clone so REST routes get their own handle for endpoint calls.
            let routes_client = c.clone();
            {
                let mut c = conn.write().await;
                c.connected = true;
                c.last_error = None;
                c.client = Some(routes_client);
            }
            c
        }
        Err(e) => {
            error!(error = %e, "Failed to connect to FC");
            conn.write().await.set_disconnected(e.to_string());
            return;
        }
    };

    // Subscribe to RecordTopic (ToClient direction: FC publishes → GS receives).
    let mut sub = match client.subscribe::<proto::RecordTopic>().await {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to subscribe to RecordTopic");
            conn.write().await.set_disconnected(format!("subscribe failed: {e}"));
            return;
        }
    };

    info!("Subscribed to RecordTopic, waiting for telemetry...");

    // Receive records until the subscription drops (FC disconnects).
    while let Some(record) = sub.recv().await {
        let mut store = storage.write().await;
        if let Some(ref mut s) = *store
            && let Err(e) = s.store_record(record)
        {
            warn!(error = %e, "Failed to write record to storage");
        }
    }

    // Subscription closed = FC disconnected.
    let msg = "FC disconnected (subscription closed)";
    warn!("{msg}");
    conn.write().await.set_disconnected(msg.to_string());
}

async fn connect_fc() -> anyhow::Result<proto::PostcardClient> {
    let client = connect_client::<{ Config::CLIENT_QUEUE_DEPTH }>(utils::constants::GS_SOCKET_NAME)
        .await
        .map_err(|e| anyhow::anyhow!("connect to {} failed: {e}", utils::constants::GS_SOCKET_NAME))?;
    Ok(client)
}
