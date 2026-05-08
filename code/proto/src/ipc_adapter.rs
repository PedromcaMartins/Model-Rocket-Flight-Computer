extern crate std;

use interprocess::local_socket::tokio::{RecvHalf, SendHalf, Stream};
use interprocess::local_socket::traits::tokio::Stream as _;
use postcard_rpc::header::{VarHeader, VarKey, VarKeyKind, VarSeq};
use postcard_rpc::server::{
    WireRx, WireRxErrorKind, WireTx, WireTxErrorKind,
};
use postcard_rpc::standard_icd::LoggingTopic;
use postcard_rpc::Topic;
use core::fmt::Arguments;
use std::string::ToString;
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

pub struct InterprocessWireTx(Mutex<SendHalf>);

pub struct InterprocessWireRx(RecvHalf);

pub fn interprocess_wire_from_stream(stream: Stream) -> (InterprocessWireTx, InterprocessWireRx) {
    let (rx, tx) = stream.split();
    (InterprocessWireTx(Mutex::new(tx)), InterprocessWireRx(rx))
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
