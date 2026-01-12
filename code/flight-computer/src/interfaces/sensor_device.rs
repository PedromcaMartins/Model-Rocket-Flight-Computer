use proto::Record;

pub trait SensorDevice {
    type Data: Into<Record> + Clone + PartialEq + core::fmt::Debug + 'static;
    type Error: core::fmt::Debug;

    const NAME: &'static str;
    const TICK_INTERVAL: embassy_time::Duration;

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error>;

    fn ticker(&self) -> embassy_time::Ticker {
        embassy_time::Ticker::every(Self::TICK_INTERVAL)
    }
}
