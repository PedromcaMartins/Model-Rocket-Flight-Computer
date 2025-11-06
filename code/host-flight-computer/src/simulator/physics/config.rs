use telemetry_messages::{Acceleration, Altitude, GpsCoordinates, Velocity};
use uom::si::{acceleration::meter_per_second_squared, f32::{Force, Mass, Time}, length::meter, time::{millisecond, second}, velocity::meter_per_second, force::newton, mass::gram};

#[derive(Copy, Clone)]
pub struct PhysicsConfig {
    pub time_step_period: Time,
    // how much faster than real time the simulation should run
    pub time_acceleration_factor: f32,
    pub gravity: Acceleration,
    pub mass: Mass,

    pub launchpad_altitude: Altitude,
    pub launchpad_coordinates: GpsCoordinates,

    pub motor_burn_time: Time,
    pub motor_avg_thrust: Force,

    pub recovery_response_time: Time,
    pub recovery_terminal_velocity: Velocity,

    pub landing_altitude: Altitude,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            time_step_period: Time::new::<millisecond>(1.0),
            time_acceleration_factor: 1.0,
            gravity: Acceleration::new::<meter_per_second_squared>(9.81),
            mass: Mass::new::<gram>(232.0),

            launchpad_coordinates: GpsCoordinates { latitude: 47.397742, longitude: 8.545594 },
            launchpad_altitude: Altitude::new::<meter>(90.0),

            motor_burn_time: Time::new::<second>(1.61),
            motor_avg_thrust: Force::new::<newton>(10.4),

            recovery_response_time: Time::new::<second>(2.0),
            recovery_terminal_velocity: Velocity::new::<meter_per_second>(5.0),

            landing_altitude: Altitude::new::<meter>(86.0),
        }
    }
}
