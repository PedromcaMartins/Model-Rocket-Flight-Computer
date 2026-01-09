use core::ops::DerefMut;

use defmt_or_log::{debug, info, error};
use embassy_time::Timer;
use postcard_rpc::{header::VarHeader, server::{Server, SpawnContext}};
use proto::{PingRequest, PingResponse};

use crate::{config::PostcardConfig, interfaces::Led};

pub struct Context {
}

pub struct SpawnCtx {
}

impl SpawnContext for Context {
    type SpawnCtxt = SpawnCtx;
    fn spawn_ctxt(&mut self) -> Self::SpawnCtxt {
        SpawnCtx{  }
    }
}

pub fn ping_handler(_context: &mut Context, _header: VarHeader, rqst: PingRequest) -> PingResponse {
    info!("ping: {}", *rqst);
    (*rqst).into()
}

pub const fn embassy_time_tick_hz_handler(_context: &mut Context, _header: VarHeader, _rqst: ()) -> u64 {
    embassy_time::TICK_HZ
}

/// This handles the server management
pub async fn postcard_server_task<Tx, Rx, Buf, D, LED>(
    mut server: Server<Tx, Rx, Buf, D>, 
    mut led: LED,
) -> !
where
    Tx: postcard_rpc::server::WireTx,
    Rx: postcard_rpc::server::WireRx,
    Buf: DerefMut<Target = [u8]>,
    D: postcard_rpc::server::Dispatch<Tx = Tx>,
    LED: Led,
{
    loop {
        if led.on().await.is_err() { error!("Postcard server: Status Led error"); }
        // If the host disconnects, we'll return an error here.
        // If this happens, just wait until the host reconnects
        let _ = server.run().await;
        if led.off().await.is_err() { error!("Postcard server: Status Led error"); }

        debug!("Postcard server disconnected, waiting for reconnect...");
        Timer::after(PostcardConfig::RECONNECT_INTERVAL).await;
    }
}
