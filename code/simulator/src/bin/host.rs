use proto::wire::connect_client;
use tokio_util::sync::CancellationToken;
use tracing::info;
use utils::logging::{LogConfig, UiConfig};
use utils::workspace;

use simulator::{connect::connect_with_retry, config::Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::logging::install_panic_hook();
    let _guard = utils::logging::init_tracing(LogConfig {
        log_root: workspace::workspace_root().join("logs"),
        stdout_level: Config::STDOUT_LOG_LEVEL,
        ui: UiConfig::TuiBuffer,
    })?;

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

    let client = connect_with_retry(utils::constants::SIM_SOCKET_NAME, || async {
        connect_client::<{ Config::CLIENT_OUTGOING_DEPTH }>(utils::constants::SIM_SOCKET_NAME)
            .await
            .map_err(anyhow::Error::from)
    }, cancel.clone()).await?;

    info!("connected to {}", utils::constants::SIM_SOCKET_NAME);

    simulator::run_simulator(client, cancel, tui_cancel).await?;
    Ok(())
}
