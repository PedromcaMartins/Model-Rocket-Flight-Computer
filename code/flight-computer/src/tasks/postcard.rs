use core::ops::DerefMut;

use crate::log::{debug, info, warn};
use embassy_time::Timer;
use postcard_rpc::{header::VarHeader, server::{Server, SpawnContext}};
use proto::{PingRequest, PingResponse};
use proto::wire::GlobalTickHz;

use crate::{config::PostcardConfig, interfaces::Led};

#[derive(Default)]
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

pub fn embassy_time_tick_hz_handler(_context: &mut Context, _header: VarHeader, _rqst: ()) -> GlobalTickHz {
    GlobalTickHz::set_global_tick_hz(embassy_time::TICK_HZ)
}

/// Handles the server management for GS connections.
/// On disconnect, waits and reconnects (GS is observational).
///
/// This variant assumes the underlying wire reconnects below the postcard
/// layer (e.g. USB on PIL/HW), so re-entering `server.run()` is meaningful.
/// For one-shot transports, use [`postcard_server_task_oneshot`] inside 
/// an outer accept loop instead.
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
        led.on().await.unwrap_or_else(|e| warn!("Postcard server: Status Led error: {:?}", e));
        let _ = server.run().await;
        led.off().await.unwrap_or_else(|e| warn!("Postcard server: Status Led error: {:?}", e));
        debug!("Postcard server disconnected, waiting for reconnect...");
        Timer::after(PostcardConfig::RECONNECT_INTERVAL).await;
    }
}

/// One-shot postcard server task: runs the server until the underlying wire
/// disconnects, then returns; the caller owns the outer accept loop.
pub async fn postcard_server_task_oneshot<Tx, Rx, Buf, D, LED>(
    mut server: Server<Tx, Rx, Buf, D>,
    mut led: LED,
)
where
    Tx: postcard_rpc::server::WireTx,
    Rx: postcard_rpc::server::WireRx,
    Buf: DerefMut<Target = [u8]>,
    D: postcard_rpc::server::Dispatch<Tx = Tx>,
    LED: Led,
{
    led.on().await.unwrap_or_else(|e| warn!("Postcard server: Status Led error: {:?}", e));
    let _ = server.run().await;
    led.off().await.unwrap_or_else(|e| warn!("Postcard server: Status Led error: {:?}", e));
    debug!("Postcard server disconnected");
}

