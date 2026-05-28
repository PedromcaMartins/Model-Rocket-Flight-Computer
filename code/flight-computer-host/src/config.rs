use std::time::Duration;

pub struct Config;
impl Config {
    pub const SERVER_BUFFER_SIZE: usize = 8 * 1024; // Bytes

    pub const GS_ACCEPT_RETRY_INTERVAL: Duration = Duration::from_secs(1);

    pub const STDOUT_LOG_LEVEL: tracing::level_filters::LevelFilter =
        utils::constants::STDOUT_LOG_LEVEL;
}
