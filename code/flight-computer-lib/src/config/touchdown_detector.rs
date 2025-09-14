use embassy_time::Duration;
use telemetry_messages::{Altitude, Velocity};
use uom::si::{length::meter, velocity::meter_per_second};

#[derive(Copy, Clone)]
pub struct TouchdownDetectorConfig {
    pub touchdown_stability_threshold: Altitude,
    pub touchdown_velocity_threshold: Velocity,
    pub detector_tick_period: Duration,
}

impl TouchdownDetectorConfig {
    pub const ALTITUDE_BUFFER_SIZE: usize = 10;
    pub const VELOCITY_BUFFER_SIZE: usize = 10;
}

impl Default for TouchdownDetectorConfig {
    fn default() -> Self {
        Self {
            touchdown_stability_threshold: Altitude::new::<meter>(1.0),
            touchdown_velocity_threshold: Velocity::new::<meter_per_second>(0.5),
            detector_tick_period: Duration::from_millis(100),
        }
    }
}
