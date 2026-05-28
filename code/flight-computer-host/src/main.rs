//! Host-side FC binary — binds two local sockets, accepts the simulator and GS
//! backend, then hands both postcard-rpc servers to the FC library.
//!
//! **Socket layout:**
//! - `fc-sim.sock` — simulator connects here (blocks until accepted).
//! - `fc-gs.sock` — GS backend connects here (accepted in background loop with
//!   retry on disconnect).
//!
//! The GS-accept loop never blocks the FC ↔ simulator loop — a missing or
//! restarting GS is tolerated (M2.2 design constraint).
//!
//! See [`README.md`](README.md) for the crate overview.

mod config;
mod dispatch;

use std::sync::Arc;

use config::Config;
use dispatch::{gs, sim};
use postcard_rpc::server::impls::test_channels::ChannelWireSpawn;
use flight_computer::tasks::{postcard::Context, simulation::start_host_flight_computer};
use proto::wire::{accept_server, bind_listener};
use tracing::{info, warn};
use utils::logging::{LogConfig, UiConfig};
use utils::workspace;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::logging::install_panic_hook();
    let _guard = utils::logging::init_tracing(LogConfig {
        log_root: workspace::workspace_root().join("logs"),
        stdout_level: Config::STDOUT_LOG_LEVEL,
        ui: UiConfig::Stdout,
    })?;

    // Bind both listeners up front so clients can connect immediately.
    let sim_listener = bind_listener(utils::constants::SIM_SOCKET_NAME)?;
    let gs_listener = Arc::new(bind_listener(utils::constants::GS_SOCKET_NAME)?);

    // Accept the simulator first (blocking — the FC has nothing to do until
    // sensor data arrives).
    info!("Waiting for simulator on {}...", utils::constants::SIM_SOCKET_NAME);
    let sim_dispatch = sim::SimDispatch::new(Context::default(), ChannelWireSpawn);
    let sim_server =
        accept_server::<{ Config::SERVER_BUFFER_SIZE }, _, _>(&sim_listener, sim_dispatch, vec![0u8; Config::SERVER_BUFFER_SIZE]).await?;
    info!("Simulator connected on {}", utils::constants::SIM_SOCKET_NAME);

    // GS backend factory: each call returns a Future that resolves to a
    // freshly accepted GS server. Transient accept errors are logged and
    // retried — the FC must never panic mid-flight on a GS hiccup, since
    // GS is observational (M2.2). The factory is invoked once per
    // (re)connect by `start_host_flight_computer`'s GS subsystem loop.
    let gs_server_factory = move || {
        let gs_listener = gs_listener.clone();
        async move {
            info!("Waiting for GS backend on {}...", utils::constants::GS_SOCKET_NAME);
            loop {
                let gs_dispatch = gs::GsDispatch::new(Context::default(), ChannelWireSpawn);
                match accept_server::<{ Config::SERVER_BUFFER_SIZE }, _, _>(
                    &gs_listener,
                    gs_dispatch,
                    vec![0u8; Config::SERVER_BUFFER_SIZE],
                ).await {
                    Ok(server) => {
                        info!("GS backend connected on {}", utils::constants::GS_SOCKET_NAME);
                        return server;
                    }
                    Err(e) => {
                        warn!("GS backend accept failed, retrying: {e:?}");
                        tokio::time::sleep(Config::GS_ACCEPT_RETRY_INTERVAL).await;
                    }
                }
            }
        }
    };

    info!("Starting flight computer (sim live, GS deferred)");
    start_host_flight_computer(sim_server, gs_server_factory).await;

    Ok(())
}
