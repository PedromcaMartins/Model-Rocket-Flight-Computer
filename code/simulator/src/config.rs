use tokio::time::Duration;

use proto::sensor_data::{Altitude, GpsCoordinates, Velocity};
use proto::uom::si::{f32::{Force, Mass, Time}, length::meter, time::second, velocity::meter_per_second, force::newton, mass::gram};
use tokio::time::{Interval, interval};

#[derive(Debug)]
pub struct SimulatorConfig {
    pub time_step: Time,
    pub time_step_interval: Interval,
    pub data_acquisition_interval: Interval,

    pub gravity: Force,
    pub rocket_mass: Mass,

    pub launchpad_altitude: Altitude,
    pub launchpad_coordinates: GpsCoordinates,

    pub motor_burn_time: Time,
    pub motor_avg_thrust: Force,

    pub recovery_response_time: Time,
    pub recovery_terminal_velocity: Velocity,

    pub landing_altitude: Altitude,
}

impl SimulatorConfig {
    pub const PHYSICS_ENGINE_TIME_STEP: f32 = 0.001;
    pub const DATA_ACQUISITION_TIME_STEP: f32 = 0.020;

    pub const SIMULATOR_COMMAND_CAPACITY: usize = 1024;
    pub const FLIGHT_COMPUTER_COMMAND_CAPACITY: usize = 1024;
    pub const PHYSICS_STATE_CAPACITY: usize = 1024;

    pub const ACTIVATION_DELAY_IGNITION: Duration = Duration::from_secs(5);
    pub const ACTIVATION_DELAY_ARM: Duration = Duration::from_secs(10);
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            time_step: Time::new::<second>(Self::PHYSICS_ENGINE_TIME_STEP),
            time_step_interval: interval(Duration::from_secs_f32(Self::PHYSICS_ENGINE_TIME_STEP)),
            data_acquisition_interval: interval(Duration::from_secs_f32(Self::DATA_ACQUISITION_TIME_STEP)),

            gravity: Force::new::<newton>(9.81),
            rocket_mass: Mass::new::<gram>(232.0),

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
