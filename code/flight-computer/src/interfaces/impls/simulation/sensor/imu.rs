use crate::{config::DataAcquisitionConfig, interfaces::{SensorDevice, impls::simulation::sensor::SimSensor}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::sensor_data::ImuData;

static LATEST_DATA: Signal<CriticalSectionRawMutex, ImuData> = Signal::new();

#[derive(Default)]
pub struct SimImu;
impl SimSensor for SimImu {
    fn signal() -> &'static Signal<CriticalSectionRawMutex, Self::Data> {
        &LATEST_DATA
    }
}

impl SensorDevice for SimImu {
    type Data = ImuData;
    type Error = ();

    const NAME: &'static str = "Simulated IMU";
    const TICK_INTERVAL: embassy_time::Duration = DataAcquisitionConfig::IMU_TICK_INTERVAL;

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error> {
        Ok(LATEST_DATA.wait().await)
    }
}
