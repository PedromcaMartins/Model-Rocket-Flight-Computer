use crate::{config::DataAcquisitionConfig, interfaces::{Sensor, impls::simulation::sensor::SimSensor}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::sensor_data::GpsData;

static LATEST_DATA: Signal<CriticalSectionRawMutex, GpsData> = Signal::new();

#[derive(Default)]
pub struct SimGps;
impl SimSensor for SimGps {
    fn signal() -> &'static Signal<CriticalSectionRawMutex, Self::Data> {
        &LATEST_DATA
    }
}

impl Sensor for SimGps {
    type Data = GpsData;
    type Error = ();

    const NAME: &'static str = "Simulated GPS";
    const TICK_INTERVAL: embassy_time::Duration = DataAcquisitionConfig::GPS_TICK_INTERVAL;

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error> {
        Ok(LATEST_DATA.wait().await)
    }
}
