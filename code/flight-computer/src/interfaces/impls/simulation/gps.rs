use crate::interfaces::SensorDevice;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::sensor_data::GpsData;

static LATEST_DATA: Signal<CriticalSectionRawMutex, GpsData> = Signal::new();

pub struct SimGps;
impl SimGps {

    pub async fn update_data(data: GpsData) {
        LATEST_DATA.signal(data);
    }
}

impl SensorDevice for SimGps {
    type Data = GpsData;
    type Error = ();

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error> {
        Ok(LATEST_DATA.wait().await)
    }
}
