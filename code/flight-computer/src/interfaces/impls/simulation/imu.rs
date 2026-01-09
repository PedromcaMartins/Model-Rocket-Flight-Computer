use crate::{config::DataAcquisitionConfig, interfaces::SensorDevice};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::sensor_data::ImuData;

static LATEST_DATA: Signal<CriticalSectionRawMutex, ImuData> = Signal::new();

pub struct SimImu;
impl SimImu {

    pub async fn update_data(data: ImuData) {
        LATEST_DATA.signal(data);
    }
}

impl SensorDevice for SimImu {
    type Data = ImuData;
    type Error = ();

    const NAME: &'static str = "Simulated IMU";
    const TICKER_PERIOD_MS: embassy_time::Duration = DataAcquisitionConfig::IMU_TICKER_PERIOD;

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error> {
        Ok(LATEST_DATA.wait().await)
    }
}
