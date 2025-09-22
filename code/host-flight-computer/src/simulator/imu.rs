use flight_computer_lib::interfaces::SensorDevice;
use telemetry_messages::ImuMessage;
use tokio::sync::mpsc;

pub struct SimImu {
    rx: mpsc::Receiver<ImuMessage>,
}

impl SimImu {
    pub fn new(rx: mpsc::Receiver<ImuMessage>) -> Self {
        Self { rx }
    }
}

impl SensorDevice for SimImu {
    type DataMessage = ImuMessage;
    type DeviceError = ();

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        self.rx.recv().await.ok_or(())
    }
}
