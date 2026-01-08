pub trait SensorDevice {
    type Data: core::fmt::Debug + proto::Serialize;
    type Error: core::fmt::Debug;

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error>;
}
