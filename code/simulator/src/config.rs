use proto::sensor_data::{Altitude, GpsCoordinates, Pressure, ThermodynamicTemperature};
use proto::uom::si::pressure::pascal;
use proto::uom::si::thermodynamic_temperature::degree_celsius;
use tokio::time::Duration;

use proto::uom::si::f32::{Acceleration, Force, Mass, Time, Velocity};
use proto::uom::si::acceleration::meter_per_second_squared;
use proto::uom::si::force::newton;
use proto::uom::si::mass::gram;
use proto::uom::si::length::meter;
use proto::uom::si::time::second;
use proto::uom::si::velocity::meter_per_second;

/// Compile-time simulator configuration.
pub struct SimulatorConfig;

impl SimulatorConfig {
    // Timing
    pub const PHYSICS_TIME_STEP_INTERVAL: Duration = Duration::from_millis(1);
    pub const DATA_ACQUISITION_INTERVAL: Duration = Duration::from_millis(20);

    // Physics
    pub fn gravity() -> Acceleration { Acceleration::new::<meter_per_second_squared>(9.81) }
    pub fn rocket_mass() -> Mass { Mass::new::<gram>(232.0) }

    // Motor
    pub fn motor_avg_thrust() -> Force { Force::new::<newton>(10.4) }
    pub fn motor_burn_time() -> Time { Time::new::<second>(1.61) }

    // Recovery
    pub fn recovery_response_time() -> Time { Time::new::<second>(2.0) }
    pub fn terminal_velocity() -> Velocity { Velocity::new::<meter_per_second>(5.0) }

    // Activation delay
    pub fn recovery_activation_delay() -> Time { Time::new::<second>(2.0) }

    // Launchpad / landing
    pub fn launchpad_altitude() -> Altitude { Altitude::new::<meter>(90.0) }
    pub const LAUNCHPAD_COORDINATES: GpsCoordinates = GpsCoordinates { latitude: 47.397742, longitude: 8.545594 };
    pub fn touch_down_altitude() -> Altitude { Altitude::new::<meter>(86.0) }

    // Scripted scenario delays (wall-clock)
    // Set to `None` to skip the corresponding event entirely.
    pub const IGNITION_DELAY: Option<Duration> = Some(Duration::from_millis(5_000));
    pub const ARM_DELAY: Option<Duration> = Some(Duration::from_millis(5_000));
    pub const ARM_ACTIVE_DELAY: Duration = Duration::from_millis(500);

    // Environment
    pub const GPS_FIX_SATELLITES: u8 = 12;
    pub fn sea_level_pressure() -> Pressure { Pressure::new::<pascal>(101_325.0) }
    pub fn ambient_temperature() -> ThermodynamicTemperature { ThermodynamicTemperature::new::<degree_celsius>(20.0) }
}

const _: () = assert!(
    SimulatorConfig::DATA_ACQUISITION_INTERVAL.as_millis() >= SimulatorConfig::PHYSICS_TIME_STEP_INTERVAL.as_millis(),
    "DATA_ACQUISITION_INTERVAL must be >= PHYSICS_TIME_STEP_INTERVAL",
);

pub struct Config;

impl Config {
    // Channel depths
    pub const SUBSCRIBE_DEPTH: usize = 1024;
    pub const FC_COMMAND_DEPTH: usize = 1024;
    pub const FORCE_EVENT_DEPTH: usize = 1024;

    // Logging
    pub const STDOUT_LOG_LEVEL: tracing::level_filters::LevelFilter =
        utils::constants::STDOUT_LOG_LEVEL;
    pub const TUI_LOG_LEVEL: &'static str = "info";

    // TUI
    pub const TUI_REFRESH_RATE: u64 = 60;
    pub const PHYSICS_PANEL_HEIGHT: u16 = 9;
    pub const EVENTS_PANEL_HEIGHT: u16 = 8;
    pub const ACTUATOR_PANEL_HEIGHT: u16 = 6;
    pub const LOG_PANEL_MIN_HEIGHT: u16 = 5;

    // Connection retry
    pub const CONNECT_MAX_ATTEMPTS: u32 = 20;
    pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(200);
    pub const CONNECT_RETRY_INTERVAL: Duration = Duration::from_millis(200);

    // Client connection
    pub const CLIENT_OUTGOING_DEPTH: usize = 64;
}
