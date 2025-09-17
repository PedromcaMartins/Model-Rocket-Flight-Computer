use flight_computer_lib::model::sensor_device::SensorDevice;
use telemetry_messages::AltimeterMessage;
use tokio::sync::mpsc;

pub struct SimAltimeter {
    rx: mpsc::Receiver<AltimeterMessage>,
}

impl SimAltimeter {
    pub fn new(rx: mpsc::Receiver<AltimeterMessage>) -> Self {
        Self { rx }
    }
}

impl SensorDevice for SimAltimeter {
    type DataMessage = AltimeterMessage;
    type DeviceError = ();

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        self.rx.recv().await.ok_or(())
    }
}
