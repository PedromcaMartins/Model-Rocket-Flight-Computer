use telemetry_messages::{Acceleration, Altitude, GpsCoordinates, Velocity};
use uom::si::{acceleration::meter_per_second_squared, f32::Time, length::meter, time::millisecond, velocity::meter_per_second};

use crate::simulator::physics::state::PhysicsState;

#[derive(Copy, Clone)]
pub struct PhysicsConfig {
    pub time_step_period: Time,
    pub gravity: Acceleration,
    pub initial_state: PhysicsState,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            time_step_period: Time::new::<millisecond>(1.0),
            gravity: Acceleration::new::<meter_per_second_squared>(9.81),
            initial_state: PhysicsState {
                altitude: Altitude::new::<meter>(10.0),
                velocity: Velocity::new::<meter_per_second>(0.0),
                acceleration: Acceleration::new::<meter_per_second_squared>(0.0),
                coordinates: GpsCoordinates {
                    latitude: 34.0522,
                    longitude: -118.2437,
                },
            }
        }
    }
}
