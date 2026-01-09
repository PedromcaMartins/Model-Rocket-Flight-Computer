use proto::Record;

pub trait SensorDevice {
    type Data: Into<Record>;
    type Error: core::fmt::Debug;

    const NAME: &'static str;
    const TICKER_PERIOD_MS: embassy_time::Duration;

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error>;

    fn ticker(&self) -> embassy_time::Ticker {
        embassy_time::Ticker::every(Self::TICKER_PERIOD_MS)
    }
}
