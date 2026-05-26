extern crate std;

use core::fmt::Arguments;
use core::ops::DerefMut;
use interprocess::local_socket::tokio::{Listener, RecvHalf, SendHalf, Stream};
use interprocess::local_socket::traits::tokio::{Listener as _, Stream as _};
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use postcard_rpc::header::{VarHeader, VarKey, VarKeyKind, VarSeq};
use postcard_rpc::server::{Dispatch, Server, WireRx, WireRxErrorKind, WireTx, WireTxErrorKind};
use postcard_rpc::standard_icd::LoggingTopic;
use postcard_rpc::Topic;
use std::string::ToString;
use std::sync::Arc;
use std::vec::Vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

fn map_tx_error(e: &std::io::Error) -> WireTxErrorKind {
    match e.kind() {
        std::io::ErrorKind::BrokenPipe
        | std::io::ErrorKind::ConnectionReset
        | std::io::ErrorKind::ConnectionAborted
        | std::io::ErrorKind::NotConnected => WireTxErrorKind::ConnectionClosed,
        std::io::ErrorKind::TimedOut => WireTxErrorKind::Timeout,
        _ => WireTxErrorKind::Other,
    }
}

fn serialize_msg<T: serde::Serialize + ?Sized>(
    hdr: VarHeader,
    msg: &T,
) -> Result<Vec<u8>, WireTxErrorKind> {
    let mut buf = hdr.write_to_vec();
    buf.extend_from_slice(&postcard::to_stdvec(msg).map_err(|_| WireTxErrorKind::Other)?);
    Ok(buf)
}

#[derive(Clone)]
pub struct InterprocessWireTx(Arc<Mutex<SendHalf>>);

pub struct InterprocessWireRx(RecvHalf);

pub fn interprocess_wire_from_stream(stream: Stream) -> (InterprocessWireTx, InterprocessWireRx) {
    let (rx, tx) = stream.split();
    (InterprocessWireTx(Arc::new(Mutex::new(tx))), InterprocessWireRx(rx))
}

impl WireTx for InterprocessWireTx {
    type Error = WireTxErrorKind;

    #[allow(clippy::future_not_send)]
    async fn send_log_fmt(
        &self,
        kkind: VarKeyKind,
        a: Arguments<'_>,
    ) -> Result<(), Self::Error> {
        let s = std::format!("{a}");
        self.send_log_str(kkind, &s).await
    }

    async fn send_log_str(&self, kkind: VarKeyKind, s: &str) -> Result<(), Self::Error> {
        let key = match kkind {
            VarKeyKind::Key1 => VarKey::Key1(LoggingTopic::TOPIC_KEY1),
            VarKeyKind::Key2 => VarKey::Key2(LoggingTopic::TOPIC_KEY2),
            VarKeyKind::Key4 => VarKey::Key4(LoggingTopic::TOPIC_KEY4),
            VarKeyKind::Key8 => VarKey::Key8(LoggingTopic::TOPIC_KEY),
        };
        let hdr = VarHeader {
            key,
            seq_no: VarSeq::Seq4(0),
        };
        self.send::<<LoggingTopic as Topic>::Message>(hdr, &s.to_string())
            .await
    }

    #[allow(clippy::future_not_send)]
    async fn send<T: serde::Serialize + ?Sized>(
        &self,
        hdr: VarHeader,
        msg: &T,
    ) -> Result<(), Self::Error> {
        let buf = serialize_msg(hdr, msg)?;
        self.send_raw(&buf).await
    }

    async fn send_raw(&self, buf: &[u8]) -> Result<(), Self::Error> {
        let len = u32::try_from(buf.len()).map_err(|_| WireTxErrorKind::Other)?;
        let mut stream = self.0.lock().await;
        stream
            .write_all(&len.to_le_bytes())
            .await
            .map_err(|e| map_tx_error(&e))?;
        stream
            .write_all(buf)
            .await
            .map_err(|e| map_tx_error(&e))?;
        drop(stream);
        Ok(())
    }
}

impl WireRx for InterprocessWireRx {
    type Error = WireRxErrorKind;

    async fn receive<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a mut [u8], Self::Error> {
        let mut len_bytes = [0u8; 4];
        self.0
            .read_exact(&mut len_bytes)
            .await
            .map_err(|_| WireRxErrorKind::ConnectionClosed)?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        let out = buf
            .get_mut(..len)
            .ok_or(WireRxErrorKind::ReceivedMessageTooLarge)?;
        self.0
            .read_exact(out)
            .await
            .map_err(|_| WireRxErrorKind::ConnectionClosed)?;
        Ok(out)
    }
}

/* --------------------------------------------------------------------------
 * Socket + transport constructors. These remove the duplicated
 * name-resolution / framing / Server boilerplate from the host binaries.
 * ----------------------------------------------------------------------- */

/// Bind a local-socket listener under the OS-conformant namespaced name.
///
/// # Errors
/// Returns an error if the name is invalid or the socket cannot be bound.
pub fn bind_listener(name: &str) -> std::io::Result<Listener> {
    ListenerOptions::new()
        .name(name.to_ns_name::<GenericNamespaced>()?)
        .create_tokio()
}

/// Connect to a local-socket server under the OS-conformant namespaced name.
///
/// # Errors
/// Returns an error if the name is invalid or the connection fails.
pub async fn connect_stream(name: &str) -> std::io::Result<Stream> {
    Stream::connect(name.to_ns_name::<GenericNamespaced>()?).await
}

/// Build a postcard-rpc `Server` over an already-connected interprocess
/// `Stream`, given a constructed dispatch table. `BUF` is the receive
/// buffer size in bytes.
#[must_use]
pub fn server_from_stream<const BUF: usize, D, Buf>(
    stream: Stream,
    dispatch: D,
    buf: Buf,
) -> Server<InterprocessWireTx, InterprocessWireRx, Buf, D>
where
    D: Dispatch<Tx = InterprocessWireTx>,
    Buf: DerefMut<Target = [u8]>,
{
    let (tx, rx) = interprocess_wire_from_stream(stream);
    let kkind = dispatch.min_key_len();
    Server::new(tx, rx, buf, dispatch, kkind)
}

/// Accept one connection on `listener` and build a postcard-rpc `Server`
/// for it. `BUF` is the receive buffer size in bytes.
///
/// # Errors
/// Returns an error if accepting the connection fails.
#[allow(clippy::future_not_send)]
pub async fn accept_server<const BUF: usize, D, Buf>(
    listener: &Listener,
    dispatch: D,
    buf: Buf,
) -> std::io::Result<Server<InterprocessWireTx, InterprocessWireRx, Buf, D>>
where
    D: Dispatch<Tx = InterprocessWireTx>,
    Buf: DerefMut<Target = [u8]>,
{
    let stream = listener.accept().await?;
    Ok(server_from_stream::<BUF, D, Buf>(stream, dispatch, buf))
}

/* --------------------------------------------------------------------------
 * Client-side wire adapter (postcard-rpc `PostcardClient` over an interprocess
 * `Stream`). Same 4-byte little-endian length-prefix framing as the
 * server-side `InterprocessWireTx`/`Rx` above.
 *
 * The whole submodule is gated on `feature = "client"` — server-only
 * transport users do not compile the host-client wire halves or the
 * `PostcardClient` wrapper.
 * ----------------------------------------------------------------------- */

#[cfg(feature = "client")]
pub use client_wire::*;

#[cfg(feature = "client")]
mod client_wire {
    extern crate std;

    use core::future::Future;
    use interprocess::local_socket::tokio::{RecvHalf, SendHalf, Stream};
    use interprocess::local_socket::traits::tokio::Stream as _;
    use postcard_rpc::header::VarSeqKind;
    use postcard_rpc::host_client::{
        HostClient, WireRx as HostWireRx, WireSpawn as HostWireSpawn, WireTx as HostWireTx,
    };
    use postcard_rpc::standard_icd::ERROR_PATH;
    use std::vec::Vec;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::PostcardClient;

    use super::connect_stream;

    /// Error type for the client-side interprocess wire.
    #[derive(Debug, thiserror::Error)]
    #[error("IPC client wire error: {0}")]
    pub struct IpcClientError(#[from] std::io::Error);

    /// Client-side transmit half.
    pub struct IpcClientWireTx {
        pub(super) tx: SendHalf,
    }

    impl IpcClientWireTx {
        async fn send_inner(&mut self, data: Vec<u8>) -> Result<(), IpcClientError> {
            let len = u32::try_from(data.len())
                .map_err(|_| std::io::Error::other("frame too large"))?;
            self.tx.write_all(&len.to_le_bytes()).await?;
            self.tx.write_all(&data).await?;
            Ok(())
        }
    }

    impl HostWireTx for IpcClientWireTx {
        type Error = IpcClientError;
        fn send(&mut self, data: Vec<u8>) -> impl Future<Output = Result<(), Self::Error>> + Send {
            self.send_inner(data)
        }
    }

    /// Client-side receive half.
    pub struct IpcClientWireRx {
        pub(super) rx: RecvHalf,
    }

    impl IpcClientWireRx {
        async fn receive_inner(&mut self) -> Result<Vec<u8>, IpcClientError> {
            let mut len_bytes = [0u8; 4];
            self.rx
                .read_exact(&mut len_bytes)
                .await
                .map_err(IpcClientError)?;
            let len = u32::from_le_bytes(len_bytes) as usize;
            let mut buf = std::vec![0u8; len];
            self.rx.read_exact(&mut buf).await.map_err(IpcClientError)?;
            Ok(buf)
        }
    }

    impl HostWireRx for IpcClientWireRx {
        type Error = IpcClientError;
        fn receive(&mut self) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send {
            self.receive_inner()
        }
    }

    /// Client-side task spawner (uses the ambient tokio runtime).
    pub struct IpcClientWireSpawn;

    impl HostWireSpawn for IpcClientWireSpawn {
        fn spawn(&mut self, fut: impl Future<Output = ()> + Send + 'static) {
            // Detach the task; dropping the JoinHandle is the documented way.
            tokio::spawn(fut);
        }
    }

    /// Build a postcard-rpc `PostcardClient` over an already-connected
    /// interprocess `Stream`. `DEPTH` is the outgoing queue depth in messages.
    #[must_use]
    pub fn client_from_stream<const DEPTH: usize>(stream: Stream) -> PostcardClient {
        let (rx, tx) = stream.split();
        PostcardClient::new(
            HostClient::new_with_wire(
                IpcClientWireTx { tx },
                IpcClientWireRx { rx },
                IpcClientWireSpawn,
                VarSeqKind::Seq4,
                ERROR_PATH,
                DEPTH,
            )
        )
    }

    /// Connect to a local-socket server and build a postcard-rpc `PostcardClient`.
    /// `DEPTH` is the outgoing queue depth in messages.
    ///
    /// # Errors
    /// Returns an error if the name is invalid or the connection fails.
    #[allow(clippy::future_not_send)]
    pub async fn connect_client<const DEPTH: usize>(
        name: &str,
    ) -> std::io::Result<PostcardClient> {
        let stream = connect_stream(name).await?;
        Ok(client_from_stream::<DEPTH>(stream))
    }
}
