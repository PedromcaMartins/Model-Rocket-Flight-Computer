use flight_computer_lib::interfaces::SensorDevice;
use telemetry_messages::GpsMessage;
use tokio::sync::mpsc;

pub struct SimGps {
    rx: mpsc::Receiver<GpsMessage>,
}

impl SimGps {
    pub fn new(rx: mpsc::Receiver<GpsMessage>) -> Self {
        Self { rx }
    }
}

impl SensorDevice for SimGps {
    type DataMessage = GpsMessage;
    type DeviceError = ();

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        Ok(self.rx.recv().await.expect("GPS channel closed"))
    }
}
