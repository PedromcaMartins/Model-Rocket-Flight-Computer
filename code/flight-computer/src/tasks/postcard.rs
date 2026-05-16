use core::ops::DerefMut;

use crate::log::{debug, info, error};
use embassy_time::Timer;
use postcard_rpc::{header::VarHeader, server::{Server, SpawnContext}};
use proto::{PingRequest, PingResponse, record::tick_hz::GlobalTickHz};

use crate::{config::PostcardConfig, interfaces::Led};

#[cfg(feature = "impl_sim")]
use postcard_rpc::server::{Sender, WireTx};

#[cfg(feature = "impl_sim")]
use proto::{actuator_data::ActuatorStatus, sensor_data::{AltimeterData, GpsData, ImuData}};

#[cfg(feature = "impl_sim")]
use crate::interfaces::impls::simulation::{arming_system::SimArming, sensor::{SimAltimeter, SimGps, SimImu, SimSensor}};

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

#[cfg(feature = "impl_sim")]
pub mod simulator {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    pub fn sim_altimeter_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: AltimeterData, _out: &Sender<Tx>) {
        SimAltimeter::update_data(data);
    }
    
    pub fn sim_gps_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: GpsData, _out: &Sender<Tx>) {
        SimGps::update_data(data);
    }
    
    pub fn sim_imu_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: ImuData, _out: &Sender<Tx>) {
        SimImu::update_data(data);
    }
    
    pub fn sim_arming_activate<Tx: WireTx>(_context: &mut Context, _header: VarHeader, _data: ActuatorStatus, _out: &Sender<Tx>) {
        SimArming::activate();
    }
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
        // If the host disconnects, we'll return an error here.
        // If this happens, just wait until the host reconnects
        let _ = server.run().await;
        led.off().await.unwrap_or_else(|e| error!("Postcard server: Status Led error: {:?}", e));

        debug!("Postcard server disconnected, waiting for reconnect...");
        Timer::after(PostcardConfig::RECONNECT_INTERVAL).await;
        led.on().await.unwrap_or_else(|e| error!("Postcard server: Status Led error: {:?}", e));
    }
}
