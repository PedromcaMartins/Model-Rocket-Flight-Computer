use embassy_time::Duration;
use proto::{Altitude, Velocity};
use proto::uom::si::{length::meter, velocity::meter_per_second};
use proto::LogDataType;

pub struct ApogeeDetectorConfig;
impl ApogeeDetectorConfig {
    pub const ALTITUDE_BUFFER_SIZE: usize = 5;
    pub const VELOCITY_BUFFER_SIZE: usize = 5;

    pub const DETECTOR_TICK_PERIOD: Duration = Duration::from_hz(2);

    #[inline]
    pub fn max_descent_velocity() -> Velocity { Velocity::new::<meter_per_second>(-1.0) }
    #[inline]
    pub fn min_apogee_altitude_above_launchpad() -> Altitude { Altitude::new::<meter>(0.0) }
}

pub struct DataAcquisitionConfig;
impl DataAcquisitionConfig {
    pub const ALTIMETER_TICKER_PERIOD: Duration = Duration::from_hz(50);
    pub const IMU_TICKER_PERIOD: Duration = Duration::from_hz(50);
    pub const GPS_TICKER_PERIOD: Duration = Duration::from_hz(50);
}

pub struct TasksConfig;
impl TasksConfig {
    pub const FLIGHT_STATE_WATCH_CONSUMERS: usize = 2;

    pub const ALTIMETER_SD_CARD_CHANNEL_DEPTH: usize = 10;
    pub const GPS_SD_CARD_CHANNEL_DEPTH: usize = 10;
    pub const IMU_SD_CARD_CHANNEL_DEPTH: usize = 10;
}

pub struct TouchdownDetectorConfig;
impl TouchdownDetectorConfig {
    pub const ALTITUDE_BUFFER_SIZE: usize = 10;
    pub const VELOCITY_BUFFER_SIZE: usize = 10;

    pub const DETECTOR_TICK_PERIOD: Duration = Duration::from_hz(1);

    #[inline]
    pub fn touchdown_stability_threshold() -> Altitude { Altitude::new::<meter>(1.0) }
    #[inline]
    pub fn touchdown_velocity_threshold() -> Velocity { Velocity::new::<meter_per_second>(0.5) }
}

pub struct LogFileSystemConfig;
impl LogFileSystemConfig {
    pub const WRITE_BUFFER_SIZE: usize = 576;

    pub const MAX_FILENAME_LENGTH: usize = 8;
    pub const MAX_UID_LENGTH: usize = Self::MAX_FILENAME_LENGTH - LogDataType::MAX_BASE_FILENAME_LENGTH;

    pub const FLUSH_FILES_TICKER_PERIOD: Duration = Duration::from_millis(500);

    pub const FNV_INDEX_MAP_SIZE: usize = {
        const fn next_power_of_two(n: usize) -> usize {
            if n <= 1 {
                2
            } else {
                1 << (usize::BITS - (n - 1).leading_zeros())
            }
        }
        const SIZE: usize = next_power_of_two(LogDataType::LENGTH);
        
        // Verify it's a power of 2
        const _: () = assert!(SIZE > 0 && SIZE.is_power_of_two());
        // Verify it's greater than LogDataType::LENGTH
        const _: () = assert!(SIZE > LogDataType::LENGTH);
        
        SIZE
    };
}
