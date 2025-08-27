pub mod gps;
pub mod bno055;
pub mod bmp280;

#[allow(async_fn_in_trait)]
pub trait SensorDevice {
    type DataMessage: core::fmt::Debug + telemetry_messages::Serialize;
    type DeviceError: core::fmt::Debug;

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError>;
}
