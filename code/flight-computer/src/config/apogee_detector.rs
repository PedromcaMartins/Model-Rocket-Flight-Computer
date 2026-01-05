use embassy_time::Duration;
use proto::{Altitude, Velocity};
use proto::uom::si::{length::meter, velocity::meter_per_second};

#[derive(Copy, Clone)]
pub struct ApogeeDetectorConfig {
    pub max_descent_velocity: Velocity,
    pub min_apogee_altitude_above_launchpad: Altitude,
    pub detector_tick_period: Duration,
}

impl ApogeeDetectorConfig {
    pub const ALTITUDE_BUFFER_SIZE: usize = 5;
    pub const VELOCITY_BUFFER_SIZE: usize = 5;
}

impl Default for ApogeeDetectorConfig {
    fn default() -> Self {
        Self {
            max_descent_velocity: Velocity::new::<meter_per_second>(-1.0),
            min_apogee_altitude_above_launchpad: Altitude::new::<meter>(10.0),
            detector_tick_period: Duration::from_hz(2),
        }
    }
}
