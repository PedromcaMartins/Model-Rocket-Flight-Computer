use std::time::Duration;

use flight_computer::tasks::postcard::Context;

pub struct LocalPostcardConfig {
    pub context: Context,
}

impl Default for LocalPostcardConfig {
    fn default() -> Self {
        Self {
            context: Context {},
        }
    }
}

impl LocalPostcardConfig {
    pub const SERVER_DEPTH: usize = 1024;
    pub const SERVER_RECEIVE_BUFFER_SIZE: usize = 1024;
}

pub struct RESTApiConfig {
    pub service_path: String,
    pub stream_path: String,
    pub sim_path: String,
}

impl Default for RESTApiConfig {
    fn default() -> Self {
        Self {
            service_path: "/api".to_string(),
            stream_path: "/api/stream".to_string(),
            sim_path: "/api/sim".to_string(),
        }
    }
}

pub struct LoggingConfig {
    pub system_log_path: PathBuf,
    pub system_json_log_level: LevelFilter,
    pub system_stdout_log_level: LevelFilter,
    pub flight_computer_log_name: &'static str,
    pub log_dir_path: PathBuf,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let ts = Local::now();
        let ts = ts.format("%Y_%m_%d_%H_%M_%S").to_string();
        let log_dir_path: PathBuf = PathBuf::from("logs").join(&ts);
        Self {
            system_log_path: log_dir_path.join("system.log"),
            system_json_log_level: LevelFilter::DEBUG,
            system_stdout_log_level: LevelFilter::INFO,
            flight_computer_log_name: "flight_computer",
            log_dir_path,
        }
    }
}

#[derive(Default)]
pub struct GroundStationConfig {
    pub postcard: LocalPostcardConfig,
    pub rest_api: RESTApiConfig,
    pub logging: LoggingConfig,
}

impl GroundStationConfig {
    pub const PING_INTERVAL: Duration = Duration::from_secs(5);
}
