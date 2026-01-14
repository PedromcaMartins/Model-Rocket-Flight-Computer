use chrono::Local;
use proto::{sensor_data::{Acceleration, AltimeterData, Altitude, AngularVelocity, GpsCoordinates, GpsData, ImuData, MagneticFluxDensity, Pressure, ThermodynamicTemperature, Time, Vector3, Velocity, nmea::sentences::FixType}, uom::si::{pressure::pascal, thermodynamic_temperature::degree_celsius}};

use crate::config::SimulatorConfig;

#[derive(Debug, Clone)]
pub struct PhysicsState {
    pub time: Time,
    pub altitude: Altitude,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub coordinates: GpsCoordinates,

    pub motor_ignited: Option<Time>,
    pub recovery_deployed: Option<Time>,
    pub landed: bool,
}

impl Default for PhysicsState {
    fn default() -> Self {
        let config = SimulatorConfig::default();

        Self {
            altitude: config.launchpad_altitude,
            coordinates: config.launchpad_coordinates,

            time: Time::default(),
            velocity: Velocity::default(),
            acceleration: Acceleration::default(),
            motor_ignited: None,
            recovery_deployed: None,
            landed: false,
        }
    }
}

impl From<PhysicsState> for AltimeterData {
    fn from(value: PhysicsState) -> Self {
        AltimeterData {
            altitude: value.altitude,
            pressure: Pressure::new::<pascal>(101325.0), // sea level standard
            temperature: ThermodynamicTemperature::new::<degree_celsius>(20.0),
        }
    }
}

impl From<PhysicsState> for ImuData {
    fn from(value: PhysicsState) -> Self {
        let gyro = AngularVelocity::default();
        let mag = MagneticFluxDensity::default();
        let accel = Acceleration::default();

        ImuData {
            acceleration: Vector3::new(accel, accel, value.acceleration),
            gyro: Vector3::new(gyro, gyro, gyro),
            mag: Vector3::new(mag, mag, mag),
            temperature: ThermodynamicTemperature::new::<degree_celsius>(20.0),
        }
    }
}

impl From<PhysicsState> for GpsData {
    fn from(value: PhysicsState) -> Self {
        GpsData {
            fix_time: Local::now().naive_local().time().into(),
            fix_type: FixType::Simulation.into(),
            coordinates: value.coordinates,
            altitude: value.altitude,
            num_of_fix_satellites: 12,
        }
    }
}
