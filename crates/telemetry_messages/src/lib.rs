#![no_std]
#![deny(unsafe_code)]

use chrono::NaiveTime;
use nmea::sentences::FixType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AltimeterMessage {
    /// Pressure in Pascal.
    pub pressure: f64,
    /// Altitude in meters.
    pub altitude: f32,
    /// Temperature in Celsius degrees.
    pub temperature: f32,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpsMessage {
    /// Timestamp
    pub fix_time: NaiveTime,
    /// Type of GPS Fix
    pub fix_type: FixType,
    /// Latitude in degrees.
    pub latitude: f64,
    /// Longitude in degrees.
    pub longitude: f64,
    /// MSL Altitude in meters
    pub altitude: f32,
    /// Number of satellites used for fix.
    pub num_of_fix_satellites: u8,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImuMessage {
    /// Euler angles representation of heading in degrees.
    /// Euler angles is represented as (`roll`, `pitch`, `yaw/heading`).
    pub euler_angles: [f32; 3],
    /// Standard quaternion represented by the scalar and vector parts. Corresponds to a right-handed rotation matrix.
    /// Quaternion is represented as (x, y, z, s).
    ///
    /// where:
    /// x, y, z: Vector part of a quaternion;
    /// s: Scalar part of a quaternion.
    pub quaternion: [f32; 4],
    /// Linear acceleration vector in m/s^2 units.
    pub linear_acceleration: [f32; 3],
    /// Gravity vector in m/s^2 units.
    pub gravity: [f32; 3],
    /// Acceleration vector in m/s^2 units.
    pub acceleration: [f32; 3],
    /// Gyroscope vector in deg/s units.
    pub gyro: [f32; 3],
    /// Magnetometer vector in uT units.
    pub mag: [f32; 3],
    /// Temperature of the chip in Celsius degrees.
    pub temperature: f32,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}
