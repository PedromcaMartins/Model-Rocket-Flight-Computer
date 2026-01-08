use chrono::Local;
use proto::{nalgebra::Quaternion, nmea::sentences::FixType, Acceleration, AltimeterData, Altitude, Angle, AngularVelocity, EulerAngles, GpsCoordinates, GpsData, ImuData, MagneticFluxDensity, Pressure, ThermodynamicTemperature, Vector3, Velocity};
use uom::si::{f32::Time, pressure::pascal, thermodynamic_temperature::degree_celsius, time::microsecond};

#[derive(Copy, Clone)]
pub struct PhysicsState {
    pub timestamp: Time,
    pub altitude: Altitude,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub coordinates: GpsCoordinates,

    pub motor_ignited_ts: Option<Time>,
    pub recovery_deployed_ts: Option<Time>,
    pub landed: bool,
}

impl From<PhysicsState> for AltimeterData {
    fn from(value: PhysicsState) -> Self {
        AltimeterData { // TODO
            altitude: value.altitude,
            pressure: Pressure::new::<pascal>(101325.0), // sea level standard
            temperature: ThermodynamicTemperature::new::<degree_celsius>(20.0),
            timestamp: value.timestamp.get::<microsecond>() as u64,
        }
    }
}

impl From<PhysicsState> for ImuData {
    fn from(value: PhysicsState) -> Self {
        let angle = Angle::default();
        let gyro = AngularVelocity::default();
        let mag = MagneticFluxDensity::default();
        let accel = Acceleration::default();

        ImuData { // TODO
            euler_angles: EulerAngles { roll: angle, pitch: angle, yaw: angle },
            quaternion: Quaternion::identity(),
            linear_acceleration: Vector3::new(accel, accel, value.acceleration),
            gravity: Vector3::new(accel, accel, value.acceleration),
            acceleration: Vector3::new(accel, accel, value.acceleration),
            gyro: Vector3::new(gyro, gyro, gyro),
            mag: Vector3::new(mag, mag, mag),
            temperature: ThermodynamicTemperature::new::<degree_celsius>(20.0),
            timestamp: value.timestamp.get::<microsecond>() as u64,
        }
    }
}

impl From<PhysicsState> for GpsData {
    fn from(value: PhysicsState) -> Self {
        GpsData { // TODO
            fix_time: Local::now().naive_local().time().into(),
            fix_type: FixType::Simulation.into(),
            coordinates: value.coordinates,
            altitude: value.altitude,
            num_of_fix_satellites: 12,
            timestamp: value.timestamp.get::<microsecond>() as u64,
        }
    }
}
