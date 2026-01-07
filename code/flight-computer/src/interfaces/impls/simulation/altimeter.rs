use crate::interfaces::SensorDevice;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::AltimeterMessage;

static LATEST_DATA: Signal<CriticalSectionRawMutex, AltimeterMessage> = Signal::new();

pub struct SimAltimeter;
impl SimAltimeter {
    pub async fn update_data(data: AltimeterMessage) {
        LATEST_DATA.signal(data);
    }
}

impl SensorDevice for SimAltimeter {
    type DataMessage = AltimeterMessage;
    type DeviceError = ();

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        Ok(LATEST_DATA.wait().await)
    }
}
