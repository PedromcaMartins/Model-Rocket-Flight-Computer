use flight_computer_lib::tasks::postcard::{ping_handler, Context};
use ground_station_backend::PostcardClient;
use postcard_rpc::{define_dispatch, header::VarSeqKind, host_client::test_channels as client, server::{Dispatch, Server, impls::test_channels::{ChannelWireRx, ChannelWireSpawn, ChannelWireTx, dispatch_impl::{Settings, WireRxBuf, WireRxImpl, WireSpawnImpl, WireTxImpl, new_server}}}};
use telemetry_messages::{PingEndpoint, ENDPOINT_LIST, TOPICS_IN_LIST, TOPICS_OUT_LIST};
use tokio::sync::mpsc;

pub type LocalServer = Server<WireTxImpl, WireRxImpl, WireRxBuf, SingleDispatcher>;

define_dispatch! {
    app: SingleDispatcher;
    spawn_fn: spawn_fn;
    tx_impl: WireTxImpl;
    spawn_impl: WireSpawnImpl;
    context: Context;

    endpoints: {
        list: ENDPOINT_LIST;

        | EndpointTy                | kind      | handler                       |
        | ----------                | ----      | -------                       |
        | PingEndpoint              | blocking  | ping_handler                  |
    };
    topics_in: {
        list: TOPICS_IN_LIST;

        | TopicTy                   | kind      | handler                       |
        | ----------                | ----      | -------                       |
    };
    topics_out: {
        list: TOPICS_OUT_LIST;
    };
}

pub fn postcard_local_setup(
    context: Context,
    server_depth: usize,
    server_receive_buffer_size: usize,
) -> (LocalServer, PostcardClient) {
    let (client_tx, server_rx) = mpsc::channel(server_depth);
    let (server_tx, client_rx) = mpsc::channel(server_depth);

    let app = SingleDispatcher::new(
        context,
        ChannelWireSpawn {},
    );

    let cwrx = ChannelWireRx::new(server_rx);
    let cwtx = ChannelWireTx::new(server_tx);

    let kkind = app.min_key_len();
    let server = new_server(
        app,
        Settings {
            tx: cwtx,
            rx: cwrx,
            buf: server_receive_buffer_size,
            kkind,
        },
    );

    let client = client::new_from_channels(client_tx, client_rx, VarSeqKind::Seq1);

    (server, client.into())
}

/// This handles the server management
pub async fn server_task(mut server: LocalServer) -> ! {
    loop {
        // If the host disconnects, we'll return an error here.
        // If this happens, just wait until the host reconnects
        let _ = server.run().await;
        tracing::debug!("Postcard server disconnected, waiting for reconnect...");
    }
}
