use std::future::Future;

use tokio_util::sync::CancellationToken;

use crate::config::Config;

/// Retry a connect function with exponential backoff, capped.
///
/// The `connect` closure is called repeatedly until it returns `Ok(...)` or the
/// cancellation token fires. Each failure is logged at `INFO` level; the final
/// failure at `MAX_ATTEMPTS` is returned as an `anyhow::Error`.
///
/// # Type parameters
///
/// - `F` — the connect closure. Must be `FnMut` so it can be called repeatedly.
/// - `Fut` — the future returned by `connect`.
/// - `T` — the successfully connected resource (typically `Postcardclient`).
///
/// # Panics
///
/// Does not panic.
pub async fn connect_with_retry<F, Fut, T>(
    label: &str,
    mut connect: F,
    cancel: CancellationToken,
) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    for attempt in 1..=Config::CONNECT_MAX_ATTEMPTS {
        if cancel.is_cancelled() {
            anyhow::bail!("connect '{label}': cancelled");
        }

        let result = tokio::time::timeout(Config::CONNECT_TIMEOUT, connect())
            .await
            .map_err(anyhow::Error::from);
        match result {
            Ok(Ok(client)) => return Ok(client),
            Ok(Err(e)) | Err(e) => {
                tracing::warn!("connect '{label}': attempt {attempt}/{} tried, retrying; error: {e}", Config::CONNECT_MAX_ATTEMPTS);
                tokio::select! {
                    _ = cancel.cancelled() => anyhow::bail!("connect '{label}': cancelled"),
                    _ = tokio::time::sleep(Config::CONNECT_RETRY_INTERVAL) => {}
                }
            }
        }
    }

    anyhow::bail!("connect '{label}': exhausted after {} attempts — is the FC host running?", Config::CONNECT_MAX_ATTEMPTS);
}
