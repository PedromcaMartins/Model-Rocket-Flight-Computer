use flight_computer::tasks::postcard::{Context, embassy_time_tick_hz_handler, ping_handler, simulator::{sim_altimeter_update, sim_arming_activate, sim_gps_update, sim_imu_update}};
use crate::{PostcardClient, LocalPostcardConfig};
use postcard_rpc::{define_dispatch, header::VarSeqKind, host_client::test_channels as client, server::{Dispatch, Server, impls::test_channels::{ChannelWireRx, ChannelWireSpawn, ChannelWireTx, dispatch_impl::{Settings, WireRxBuf, WireRxImpl, WireSpawnImpl, WireTxImpl, new_server}}}};
use proto::{ENDPOINT_LIST, GlobalTickHzEndpoint, PingEndpoint, SimAltimeterTopic, SimArmTopic, SimGpsTopic, SimImuTopic, TOPICS_IN_LIST, TOPICS_OUT_LIST};
use tokio::sync::mpsc;

pub type LocalServer = Server<WireTxImpl, WireRxImpl, WireRxBuf, App>;

define_dispatch! {
    app: App;
    spawn_fn: spawn_fn;
    tx_impl: WireTxImpl;
    spawn_impl: WireSpawnImpl;
    context: Context;

    endpoints: {
        list: ENDPOINT_LIST;

        | EndpointTy                | kind      | handler                       |
        | ----------                | ----      | -------                       |
        | PingEndpoint              | blocking  | ping_handler                  |
        | GlobalTickHzEndpoint      | blocking  | embassy_time_tick_hz_handler  |
    };
    topics_in: {
        list: TOPICS_IN_LIST;

        | TopicTy                   | kind      | handler                       |
        | ----------                | ----      | -------                       |
        | SimAltimeterTopic         | blocking  | sim_altimeter_update          |
        | SimGpsTopic               | blocking  | sim_gps_update                |
        | SimImuTopic               | blocking  | sim_imu_update                |
        | SimArmTopic               | blocking  | sim_arming_activate           |
    };
    topics_out: {
        list: TOPICS_OUT_LIST;
    };
}

pub fn postcard_local_setup(config: LocalPostcardConfig) -> (LocalServer, PostcardClient) {
    let LocalPostcardConfig {
        context,
    } = config;

    let (client_tx, server_rx) = mpsc::channel(LocalPostcardConfig::SERVER_DEPTH);
    let (server_tx, client_rx) = mpsc::channel(LocalPostcardConfig::SERVER_DEPTH);

    let app = App::new(
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
            buf: LocalPostcardConfig::SERVER_RECEIVE_BUFFER_SIZE,
            kkind,
        },
    );

    let client = client::new_from_channels(client_tx, client_rx, VarSeqKind::Seq4);

    (server, client.into())
}
