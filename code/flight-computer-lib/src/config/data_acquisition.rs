use embassy_time::Duration;

#[derive(Copy, Clone)]
pub struct DataAcquisitionConfig {
    pub altimeter_ticker_period: Duration,
    pub imu_ticker_period: Duration,
    pub gps_ticker_period: Duration,
}

impl Default for DataAcquisitionConfig {
    fn default() -> Self {
        Self {
            altimeter_ticker_period: Duration::from_hz(50),
            imu_ticker_period: Duration::from_hz(50),
            gps_ticker_period: Duration::from_hz(50),
        }
    }
}
