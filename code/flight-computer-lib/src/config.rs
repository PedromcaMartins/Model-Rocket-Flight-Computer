mod apogee_detector;
pub use apogee_detector::ApogeeDetectorConfig;

mod touchdown_detector;
pub use touchdown_detector::TouchdownDetectorConfig;

mod log_filesystem;
pub use log_filesystem::LogFileSystemConfig;

mod tasks;
pub use tasks::TasksConfig;

#[derive(Default)]
pub struct FlightComputerConfig {
    pub apogee_detector: ApogeeDetectorConfig,
    pub touchdown_detector: TouchdownDetectorConfig,
    pub log_filesystem: LogFileSystemConfig,
}
