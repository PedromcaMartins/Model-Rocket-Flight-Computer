use core::ops::DerefMut;

use defmt_or_log::{debug, info, error};
use embassy_time::Timer;
use postcard_rpc::{header::VarHeader, server::{Sender, Server, SpawnContext, WireTx}};
use proto::{PingRequest, PingResponse, actuator_data::ActuatorStatus, record::tick_hz::GlobalTickHz, sensor_data::{AltimeterData, GpsData, ImuData}};

#[cfg(feature = "impl_software")]
use crate::interfaces::impls::simulation::sensor::SimSensor;
use crate::{config::PostcardConfig, interfaces::{Led, impls::simulation::{arming_system::SimArming, sensor::{SimAltimeter, SimGps, SimImu}}}};

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

#[cfg(feature = "impl_software")]
pub mod simulator {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    pub fn sim_altimeter_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: AltimeterData, _out: &Sender<Tx>) {
        #[cfg(feature = "impl_software")]
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
        if led.off().await.is_err() { error!("Postcard server: Status Led error"); }

        debug!("Postcard server disconnected, waiting for reconnect...");
        Timer::after(PostcardConfig::RECONNECT_INTERVAL).await;
        if led.on().await.is_err() { error!("Postcard server: Status Led error"); }
    }
}
