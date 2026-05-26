use embassy_time::Duration;
use proto::sensor_data::{Altitude, Velocity};
use proto::uom::si::{length::meter, velocity::meter_per_second};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

pub struct ApogeeDetectorConfig;
impl ApogeeDetectorConfig {
    pub const ALTITUDE_BUFFER_SIZE: usize = 5;
    pub const VELOCITY_BUFFER_SIZE: usize = 5;

    const DETECTOR_TICK_INTERVAL_MS: u64 = 500;
    pub const DETECTOR_TICK_INTERVAL: Duration = Duration::from_millis(Self::DETECTOR_TICK_INTERVAL_MS);
    pub const DATA_WAIT_TIMEOUT: Duration = Duration::from_millis(Self::DETECTOR_TICK_INTERVAL_MS / 2);

    #[inline]
    pub fn max_descent_velocity() -> Velocity { Velocity::new::<meter_per_second>(-1.0) }
    #[inline]
    pub fn min_apogee_altitude_above_launchpad() -> Altitude { Altitude::new::<meter>(0.0) }
}

pub struct DataAcquisitionConfig;
impl DataAcquisitionConfig {
    pub const ALTIMETER_TICK_INTERVAL: Duration = Duration::from_hz(50);
    pub const IMU_TICK_INTERVAL: Duration = Duration::from_hz(50);
    pub const GPS_TICK_INTERVAL: Duration = Duration::from_hz(10);
}

pub struct TasksConfig;
impl TasksConfig {
    pub const FLIGHT_STATE_WATCH_CONSUMERS: usize = 5;

    pub const RECORD_TO_STORAGE_CHANNEL_DEPTH: usize = 30;
}

pub struct TouchdownDetectorConfig;
impl TouchdownDetectorConfig {
    pub const ALTITUDE_BUFFER_SIZE: usize = 10;
    pub const VELOCITY_BUFFER_SIZE: usize = 10;

    const DETECTOR_TICK_INTERVAL_MS: u64 = 1000;
    pub const DETECTOR_TICK_INTERVAL: Duration = Duration::from_millis(Self::DETECTOR_TICK_INTERVAL_MS);
    pub const DATA_WAIT_TIMEOUT: Duration = Duration::from_millis(Self::DETECTOR_TICK_INTERVAL_MS / 2);

    #[inline]
    pub fn touchdown_stability_threshold() -> Altitude { Altitude::new::<meter>(1.0) }
    #[inline]
    pub fn touchdown_velocity_threshold() -> Velocity { Velocity::new::<meter_per_second>(0.5) }
}

pub struct StorageConfig;
impl StorageConfig {
    pub const WRITE_BUFFER_SIZE: usize = 576;
    pub const MAX_FILENAME_LENGTH: usize = 8;
    pub const SD_VOLUME_IDX: usize = 0;

    pub const FLUSH_FILES_TICK_INTERVAL: Duration = Duration::from_millis(500);
    pub const TOUCHDOWN_HOLD_DURATION: Duration = Duration::from_secs(30);

    pub const WRITE_TIMEOUT: Duration = DEFAULT_TIMEOUT;
    pub const FLUSH_TIMEOUT: Duration = DEFAULT_TIMEOUT;
}

pub struct GroundStationConfig;
impl GroundStationConfig {
    pub const SEND_SENSOR_DATA_TICK_INTERVAL: Duration = Duration::from_hz(10);

    pub const PUBLISH_TIMEOUT: Duration = DEFAULT_TIMEOUT;
}

pub struct PostcardConfig;
impl PostcardConfig {
    pub const RECONNECT_INTERVAL: Duration = DEFAULT_TIMEOUT;
}

pub struct FiniteStateMachineConfig;
impl FiniteStateMachineConfig {
    pub const WAITING_ARM_INTERVAL: Duration = Duration::from_hz(10);
}

pub struct ArmedConfig;
impl ArmedConfig {
    pub const DEPLOY_TIMEOUT: Duration = Duration::from_secs(1);
    pub const VERIFY_TIMEOUT: Duration = Duration::from_millis(500);
}

pub struct AltimeterConfig;
impl AltimeterConfig {
    pub const REFERENCE_PRESSURE: f32 = 101_325.0;
}

#[cfg(feature = "impl_embedded")]
pub mod embedded {
    use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby};
    use bno055::{BNO055OperationMode, BNO055PowerMode};
    use nmea::SentenceType;

    pub struct Bmp280Config;
    impl Bmp280Config {
        pub const CONFIG: Config = Config {
            filter: Filter::c16,
            t_sb: Standby::ms0_5,
        };
        pub const CONTROL: Control = Control {
            osrs_t: Oversampling::x1,
            osrs_p: Oversampling::x4,
            mode: PowerMode::Normal,
        };
    }

    pub struct Bno055Config;
    impl Bno055Config {
        pub const STARTUP_DELAY: embassy_time::Instant = embassy_time::Instant::from_millis(650);
        pub const OPERATION_MODE: BNO055OperationMode = BNO055OperationMode::NDOF;
        pub const POWER_MODE: BNO055PowerMode = BNO055PowerMode::NORMAL;
        pub const USE_EXTERNAL_CRYSTAL: bool = true;
    }

    pub struct GpsConfig;
    impl GpsConfig {
        pub const NMEA_SENTENCES_FOR_NAVIGATION: &'static [SentenceType] = &[
            SentenceType::GGA,
        ];
    }
}

#[cfg(feature = "impl_host")]
pub mod host {
    pub struct HostConfig;

    impl HostConfig {
        pub const STORAGE_PATH: &str = "host_storage";
    }
}
