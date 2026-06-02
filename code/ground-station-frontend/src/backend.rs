use std::{pin::Pin, time::Duration};
use std::task::Poll;

use futures_util::Stream;
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use tokio_tungstenite::connect_async;
use tracing::warn;

use crate::config::Config;
pub use utils::status::{PingSuccess, Status, WsMessage};

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

#[allow(async_fn_in_trait)]
pub trait BackendClient: Send + Sync {
    /// Open a WebSocket connection and return a stream of messages.
    async fn connect_ws(&self) -> anyhow::Result<WsStreamImpl>;

    /// POST /api/commands/arm
    async fn arm(&self) -> anyhow::Result<()>;

    /// POST /api/commands/ignite
    async fn ignite(&self) -> anyhow::Result<()>;

    /// POST /api/commands/ping — returns latency in ms
    async fn ping(&self) -> anyhow::Result<Duration>;

}

// ---------------------------------------------------------------------------
// Concrete backend: WsBackend
// ---------------------------------------------------------------------------

pub struct WsBackend {
    http: HttpClient,
}

impl WsBackend {
    pub fn new() -> Self {
        Self {
            http: HttpClient::new(),
        }
    }

    /// POST to `url`, return `Ok(T)` on success or `bail!` with the
    /// server's `{ "error": "..." }` message on failure.
    async fn post_json<T: DeserializeOwned>(&self, url: String) -> anyhow::Result<T> {
        let resp = self.http.post(&url).send().await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body: serde_json::Value = resp.json().await?;
            let msg = body["error"].as_str().unwrap_or("request failed");
            anyhow::bail!("{}", msg)
        }
    }
}

impl Default for WsBackend {
    fn default() -> Self {
        Self::new()
    }
}


impl BackendClient for WsBackend {
    async fn connect_ws(&self) -> anyhow::Result<WsStreamImpl> {
        let (ws_stream, _) = connect_async(Config::ws_url()).await?;
        Ok(WsStreamImpl { inner: ws_stream })
    }

    async fn arm(&self) -> anyhow::Result<()> {
        self.post_json(Config::arm_url()).await
    }

    async fn ignite(&self) -> anyhow::Result<()> {
        self.post_json(Config::ignite_url()).await
    }

    async fn ping(&self) -> anyhow::Result<Duration> {
        self.post_json::<PingSuccess>(Config::ping_url())
            .await
            .map(|p| p.latency)
    }
}

// ---------------------------------------------------------------------------
// Concrete stream type
// ---------------------------------------------------------------------------

type TungsteniteStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Concrete stream over parsed `WsMessage`s from a WebSocket connection.
/// All fields are `Unpin`, so `Pin` projection is safe without `unsafe`.
pub struct WsStreamImpl {
    inner: TungsteniteStream,
}

impl Stream for WsStreamImpl {
    type Item = WsMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        // SAFETY: `inner` is `Unpin`, so `Pin::new` is safe here.
        let mut inner = Pin::new(&mut self.get_mut().inner);
        loop {
            match inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text)))) => {
                    if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                        return Poll::Ready(Some(msg));
                    }
                    warn!("Failed to deserialize WS message");
                    continue;
                }
                Poll::Ready(Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_)))) => {
                    return Poll::Ready(None);
                }
                Poll::Ready(Some(Ok(_))) => continue, // ping/pong/binary/frame
                Poll::Ready(Some(Err(_))) => return Poll::Ready(None),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}


