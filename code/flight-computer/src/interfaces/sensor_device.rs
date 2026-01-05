pub trait SensorDevice {
    type DataMessage: core::fmt::Debug + proto::Serialize;
    type DeviceError: core::fmt::Debug;

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError>;
}
