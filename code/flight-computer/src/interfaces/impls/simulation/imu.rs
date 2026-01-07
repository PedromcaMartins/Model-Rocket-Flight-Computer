use crate::interfaces::SensorDevice;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use proto::ImuMessage;

static LATEST_DATA: Signal<CriticalSectionRawMutex, ImuMessage> = Signal::new();

pub struct SimImu;
impl SimImu {

    pub async fn update_data(data: ImuMessage) {
        LATEST_DATA.signal(data);
    }
}

impl SensorDevice for SimImu {
    type DataMessage = ImuMessage;
    type DeviceError = ();

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        Ok(LATEST_DATA.wait().await)
    }
}
