#![no_std]

use chrono::{NaiveDate, NaiveTime};
use nmea::sentences::FixType;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
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
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct AltimeterMessage {
    /// Pressure in Pascal.
    pub pressure: f64,
    /// Altitude in meters.
    pub altitude: f32,
    /// Temperature in Celsius degrees.
    pub temperature: f32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct GpsMessage {
    #[cfg_attr(feature = "defmt-03", defmt(Debug2Format))]
    pub fix_time: NaiveTime,
    #[cfg_attr(feature = "defmt-03", defmt(Debug2Format))]
    pub fix_type: FixType,
    /// Latitude in degrees.
    pub latitude: f64,
    /// Longitude in degrees.
    pub longitude: f64,
    /// MSL Altitude in meters
    pub altitude: f32,
    /// Number of satellites used for fix.
    pub num_of_fix_satellites: u8,
}
