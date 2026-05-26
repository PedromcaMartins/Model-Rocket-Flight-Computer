use proto::wire::connect_client;
use tokio_util::sync::CancellationToken;
use tracing::info;

use simulator::{connect::connect_with_retry, logging};
use simulator::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::install_panic_hook();
    let _guard = logging::init_tracing()?;

    // Two cancellation tokens: `cancel` stops core sim tasks (physics, FC, scripted),
    // `tui_cancel` signals the TUI loop. Separate so the TUI can outlive the core
    // in degraded mode (FC disconnect, physics crash) — the user can inspect final
    // state and press q to quit. Ctrl-C cancels both at once.
    let cancel = CancellationToken::new();
    let tui_cancel = CancellationToken::new();
    let cancel_ctrlc = cancel.clone();
    let tui_cancel_ctrlc = tui_cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Ctrl-C received, shutting down");
            cancel_ctrlc.cancel();
            tui_cancel_ctrlc.cancel();
        }
    });

    let client = connect_with_retry(Config::SIM_SOCKET_PATH, || async {
        connect_client::<{ Config::CLIENT_OUTGOING_DEPTH }>(Config::SIM_SOCKET_PATH)
            .await
            .map_err(anyhow::Error::from)
    }, cancel.clone()).await?;

    info!("connected to {}", Config::SIM_SOCKET_PATH);

    simulator::run_simulator(client, cancel, tui_cancel).await?;
    Ok(())
}
