use crate::interfaces::SensorDevice;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::AltimeterMessage;

pub struct SimAltimeter;
impl SimAltimeter {
    const LATEST_DATA: Signal<CriticalSectionRawMutex, AltimeterMessage> = Signal::new();

    pub async fn update_data(data: AltimeterMessage) {
        Self::LATEST_DATA.signal(data);
    }
}

impl SensorDevice for SimAltimeter {
    type DataMessage = AltimeterMessage;
    type DeviceError = ();

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        Ok(Self::LATEST_DATA.wait().await)
    }
}
