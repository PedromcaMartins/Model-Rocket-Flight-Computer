use std::time::Duration;

use tracing::level_filters::LevelFilter;

pub struct Config;
impl Config {
    pub const SIM_SOCKET_PATH: &'static str = "fc-sim.sock";
    pub const GS_SOCKET_PATH: &'static str = "fc-gs.sock";
    pub const SERVER_BUFFER_SIZE: usize = 8 * 1024; // Bytes

    pub const GS_ACCEPT_RETRY_INTERVAL: Duration = Duration::from_secs(1);

    pub const STDOUT_LOG_LEVEL: LevelFilter = LevelFilter::INFO;
    pub const LOG_ROOT_DIR: &'static str = "logs";
    pub const LOG_TIMESTAMP_FORMAT: &'static str = "%Y_%m_%d_%H_%M_%S";
}
