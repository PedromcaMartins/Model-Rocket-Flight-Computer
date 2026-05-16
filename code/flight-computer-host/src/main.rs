mod config;
mod dispatch;
mod logging;

use config::Config;
use dispatch::{gs, sim};
use postcard_rpc::server::{Dispatch, Server, impls::test_channels::ChannelWireSpawn};
use flight_computer::tasks::{postcard::Context, simulation::start_host_flight_computer};
use interprocess::local_socket::traits::tokio::Listener as _;
use proto::ipc_adapter::{interprocess_wire_from_stream, InterprocessWireRx, InterprocessWireTx};
use tracing::info;

// Binds two local sockets, accepts one connection on each (simulator first,
// then GS backend), then hands both to `start_host_flight_computer` which
// joins the resulting server tasks alongside peripheral and FSM tasks.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::install_panic_hook();
    let _guard = logging::init_tracing()?;

    // Bind both listener sockets before accepting — allows clients to connect
    // immediately once we start accepting.
    let sim_listener = bind_socket(Config::SIM_SOCKET_PATH)?;
    let gs_listener = bind_socket(Config::GS_SOCKET_PATH)?;

    // Accept simulator connection first (blocking).
    // In M2+ the developer starts all three processes: FC, simulator, GS backend.
    info!("Waiting for simulator on {}...", Config::SIM_SOCKET_PATH);
    let sim_stream = sim_listener.accept().await?;
    info!("Simulator connected on {}", Config::SIM_SOCKET_PATH);

    // Accept GS backend connection second.
    info!("Waiting for GS backend on {}...", Config::GS_SOCKET_PATH);
    let gs_stream = gs_listener.accept().await?;
    info!("GS backend connected on {}", Config::GS_SOCKET_PATH);

    // Split each connection into TX/RX halves for the postcard-rpc Server.
    let (sim_tx, sim_rx) = interprocess_wire_from_stream(sim_stream);
    let (gs_tx, gs_rx) = interprocess_wire_from_stream(gs_stream);

    // Create the dispatch tables — each defines what endpoints/topics its
    // socket can handle. Both share a Context and use ChannelWireSpawn +
    // tokio_spawn from postcard-rpc (handlers are all blocking for now, but
    // tokio_spawn properly spawns any future added later).
    let sim_dispatch = sim::SimDispatch::new(Context::default(), ChannelWireSpawn);
    let gs_dispatch = gs::GsDispatch::new(Context::default(), ChannelWireSpawn);

    // Query the minimum key length each dispatch needs to avoid hash
    // collisions (depends on the number of registered endpoints/topics).
    let sim_kkind = sim_dispatch.min_key_len();
    let gs_kkind = gs_dispatch.min_key_len();

    // Allocate receive buffers.
    let sim_buf = vec![0u8; Config::SERVER_BUFFER_SIZE].into_boxed_slice();
    let gs_buf = vec![0u8; Config::SERVER_BUFFER_SIZE].into_boxed_slice();

    // Construct the two postcard-rpc servers.
    let sim_server: Server<InterprocessWireTx, InterprocessWireRx, Box<[u8]>, sim::SimDispatch> =
        Server::new(sim_tx, sim_rx, sim_buf, sim_dispatch, sim_kkind);
    let gs_server: Server<InterprocessWireTx, InterprocessWireRx, Box<[u8]>, gs::GsDispatch> =
        Server::new(gs_tx, gs_rx, gs_buf, gs_dispatch, gs_kkind);

    info!("Starting flight computer with FC-sim and FC-GS servers");
    start_host_flight_computer(sim_server, gs_server).await;

    Ok(())
}

// Create a local-socket listener.
// 
// The caller provides a bare name like "fc-sim.sock" and this function
// handles the rest.
fn bind_socket(
    name: &str,
) -> std::io::Result<interprocess::local_socket::tokio::Listener> {
    use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};

    ListenerOptions::new()
        .name(name.to_ns_name::<GenericNamespaced>()?)
        .create_tokio()
}
