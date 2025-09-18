#[derive(Default)]
pub struct TasksConfig;

impl TasksConfig {
    pub const FLIGHT_STATE_WATCH_CONSUMERS: usize = 2;

    pub const ALTIMETER_SD_CARD_CHANNEL_DEPTH: usize = 10;
    pub const GPS_SD_CARD_CHANNEL_DEPTH: usize = 10;
    pub const IMU_SD_CARD_CHANNEL_DEPTH: usize = 10;
}
