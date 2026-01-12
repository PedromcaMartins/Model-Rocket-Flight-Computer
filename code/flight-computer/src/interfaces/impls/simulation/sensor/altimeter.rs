use crate::{config::DataAcquisitionConfig, interfaces::{Sensor, impls::simulation::sensor::SimSensor}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::sensor_data::AltimeterData;

static LATEST_DATA: Signal<CriticalSectionRawMutex, AltimeterData> = Signal::new();

#[derive(Default)]
pub struct SimAltimeter;
impl SimSensor for SimAltimeter {
    fn signal() -> &'static Signal<CriticalSectionRawMutex, Self::Data> {
        &LATEST_DATA
    }
}

impl Sensor for SimAltimeter {
    type Data = AltimeterData;
    type Error = ();

    const NAME: &'static str = "Simulated Altimeter";
    const TICK_INTERVAL: embassy_time::Duration = DataAcquisitionConfig::ALTIMETER_TICK_INTERVAL;

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error> {
        Ok(LATEST_DATA.wait().await)
    }
}
