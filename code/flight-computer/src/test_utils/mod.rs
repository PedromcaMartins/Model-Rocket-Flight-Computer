#![allow(dead_code)]
#![allow(clippy::must_use_candidate)]

use rstest::fixture;

pub mod sensor_data;

pub struct TestConfig;
impl TestConfig {
}

pub fn ms(ms: u32) -> std::time::Duration {
    std::time::Duration::from_millis(ms.into())
}

#[fixture]
pub fn mock_driver() -> &'static embassy_time::MockDriver {
    embassy_time::MockDriver::get()
}

#[fixture]
pub fn mock_logger() -> logtest::Logger {
    logtest::Logger::start()
}
