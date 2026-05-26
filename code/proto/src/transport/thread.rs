extern crate std;

use core::ops::DerefMut;

use postcard_rpc::{
    header::{VarKeyKind, VarSeqKind},
    host_client::test_channels as client,
    server::{
        Dispatch, Server,
        impls::test_channels::{ChannelWireRx, ChannelWireTx},
    },
};
use tokio::sync::mpsc;

use crate::PostcardClient;

/// Create an in-process postcard-rpc server + client pair over tokio mpsc
/// channels.
///
/// The caller provides the dispatch table (built via `define_dispatch!`),
/// a receive buffer, the channel depth, and the client's `VarSeqKind`.
///
/// Returns a `(Server, PostcardClient)` pair wired together by mpsc
/// channels. The second tuple element is the same [`crate::PostcardClient`]
/// wrapper that [`crate::transport::ipc::connect_client`] returns, so
/// callers can swap transports without changing call-site code.
pub fn create_pair<D, Buf>(
    dispatch: D,
    buf: Buf,
    depth: usize,
    client_kkind: VarSeqKind,
) -> (Server<ChannelWireTx, ChannelWireRx, Buf, D>, PostcardClient)
where
    D: Dispatch<Tx = ChannelWireTx>,
    Buf: DerefMut<Target = [u8]>,
{
    let (client_tx, server_rx) = mpsc::channel(depth);
    let (server_tx, client_rx) = mpsc::channel(depth);

    let cwrx = ChannelWireRx::new(server_rx);
    let cwtx = ChannelWireTx::new(server_tx);

    let kkind: VarKeyKind = dispatch.min_key_len();
    let server = Server::new(cwtx, cwrx, buf, dispatch, kkind);
    let host_client = client::new_from_channels(client_tx, client_rx, client_kkind);

    (server, PostcardClient::new(host_client))
}
