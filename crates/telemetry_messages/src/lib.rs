#![no_std]
#![deny(unsafe_code)]

use chrono::NaiveTime;
use nalgebra::{UnitQuaternion, Vector3};
use nmea::sentences::FixType;
use uom::si::quantities::{Acceleration, Angle, AngularVelocity, Length, MagneticFluxDensity, Pressure, ThermodynamicTemperature, Time};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AltimeterMessage {
    /// Pressure in Pascal.
    pub pressure: Pressure<f64>,
    /// Altitude in meters.
    pub altitude: Length<f32>,
    /// Temperature in Celsius degrees.
    pub temperature: ThermodynamicTemperature<f32>,
    /// Timestamp in microseconds.
    pub timestamp: Time<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpsMessage {
    /// Timestamp
    pub fix_time: NaiveTime,
    /// Type of GPS Fix
    pub fix_type: FixType,
    /// Latitude in degrees.
    pub latitude: Angle<f64>,
    /// Longitude in degrees.
    pub longitude: Angle<f64>,
    /// MSL Altitude in meters
    pub altitude: Length<f32>,
    /// Number of satellites used for fix.
    pub num_of_fix_satellites: u8,
    /// Timestamp in microseconds.
    pub timestamp: Time<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImuMessage {
    /// Euler angles representation of heading in degrees.
    /// Euler angles is represented as (`roll`, `pitch`, `yaw/heading`).
    pub euler_angles: Vector3<Angle<f32>>,
    /// Standard quaternion represented by the scalar and vector parts. Corresponds to a right-handed rotation matrix.
    /// Quaternion is represented as (x, y, z, s).
    ///
    /// where:
    /// x, y, z: Vector part of a quaternion;
    /// s: Scalar part of a quaternion.
    pub quaternion: UnitQuaternion<f32>,
    /// Linear acceleration vector in m/s^2 units.
    pub linear_acceleration: Vector3<Acceleration<f32>>,
    /// Gravity vector in m/s^2 units.
    pub gravity: Vector3<Acceleration<f32>>,
    /// Acceleration vector in m/s^2 units.
    pub acceleration: Vector3<Acceleration<f32>>,
    /// Gyroscope vector in deg/s units.
    pub gyro: Vector3<AngularVelocity<f32>>,
    /// Magnetometer vector in uT units.
    pub mag: Vector3<MagneticFluxDensity<f32>>,
    /// Temperature of the chip in Celsius degrees.
    pub temperature: ThermodynamicTemperature<f32>,
    /// Timestamp in microseconds.
    pub timestamp: Time<f64>,
}
