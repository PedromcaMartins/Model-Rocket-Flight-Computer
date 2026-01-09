use crate::{config::DataAcquisitionConfig, interfaces::SensorDevice};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::sensor_data::AltimeterData;

static LATEST_DATA: Signal<CriticalSectionRawMutex, AltimeterData> = Signal::new();

pub struct SimAltimeter;
impl SimAltimeter {
    pub async fn update_data(data: AltimeterData) {
        LATEST_DATA.signal(data);
    }
}

impl SensorDevice for SimAltimeter {
    type Data = AltimeterData;
    type Error = ();

    const NAME: &'static str = "Simulated Altimeter";
    const TICKER_PERIOD_MS: embassy_time::Duration = DataAcquisitionConfig::ALTIMETER_TICKER_PERIOD;

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error> {
        Ok(LATEST_DATA.wait().await)
    }
}
