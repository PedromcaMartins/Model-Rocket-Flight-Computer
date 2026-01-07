use crate::interfaces::SensorDevice;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::GpsMessage;

static LATEST_DATA: Signal<CriticalSectionRawMutex, GpsMessage> = Signal::new();

pub struct SimGps;
impl SimGps {

    pub async fn update_data(data: GpsMessage) {
        LATEST_DATA.signal(data);
    }
}

impl SensorDevice for SimGps {
    type DataMessage = GpsMessage;
    type DeviceError = ();

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        Ok(LATEST_DATA.wait().await)
    }
}
