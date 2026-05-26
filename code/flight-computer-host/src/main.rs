mod config;
mod dispatch;
mod logging;

use std::sync::Arc;

use config::Config;
use dispatch::{gs, sim};
use postcard_rpc::server::impls::test_channels::ChannelWireSpawn;
use flight_computer::tasks::{postcard::Context, simulation::start_host_flight_computer};
use proto::wire::{accept_server, bind_listener};
use tracing::{info, warn};

// Binds both local sockets, accepts the simulator (required, blocking), then
// hands the FC a factory that accepts the GS backend in the background and
// retries on disconnect — a missing or restarting GS never blocks the
// FC <-> simulator loop.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::install_panic_hook();
    let _guard = logging::init_tracing()?;

    // Bind both listeners up front so clients can connect immediately.
    let sim_listener = bind_listener(Config::SIM_SOCKET_PATH)?;
    let gs_listener = Arc::new(bind_listener(Config::GS_SOCKET_PATH)?);

    // Accept the simulator first (blocking — the FC has nothing to do until
    // sensor data arrives).
    info!("Waiting for simulator on {}...", Config::SIM_SOCKET_PATH);
    let sim_dispatch = sim::SimDispatch::new(Context::default(), ChannelWireSpawn);
    let sim_server =
        accept_server::<{ Config::SERVER_BUFFER_SIZE }, _, _>(&sim_listener, sim_dispatch, vec![0u8; Config::SERVER_BUFFER_SIZE]).await?;
    info!("Simulator connected on {}", Config::SIM_SOCKET_PATH);

    // GS backend factory: each call returns a Future that resolves to a
    // freshly accepted GS server. Transient accept errors are logged and
    // retried — the FC must never panic mid-flight on a GS hiccup, since
    // GS is observational (M2.2). The factory is invoked once per
    // (re)connect by `start_host_flight_computer`'s GS subsystem loop.
    let gs_server_factory = move || {
        let gs_listener = gs_listener.clone();
        async move {
            info!("Waiting for GS backend on {}...", Config::GS_SOCKET_PATH);
            loop {
                let gs_dispatch = gs::GsDispatch::new(Context::default(), ChannelWireSpawn);
                match accept_server::<{ Config::SERVER_BUFFER_SIZE }, _, _>(
                    &gs_listener,
                    gs_dispatch,
                    vec![0u8; Config::SERVER_BUFFER_SIZE],
                ).await {
                    Ok(server) => {
                        info!("GS backend connected on {}", Config::GS_SOCKET_PATH);
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
