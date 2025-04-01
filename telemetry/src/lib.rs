#![no_std]

use mint::{EulerAngles, Quaternion, Vector3};
use chrono::{NaiveDate, NaiveTime};
use nmea::sentences::FixType;

pub struct ImuMessage {
    pub euler_angles: EulerAngles<f32, ()>,
    pub quaternion: Quaternion<f32>,
    pub linear_acceleration: Vector3<f32>,
    pub gravity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub gyro: Vector3<f32>,
    pub mag: Vector3<f32>,
    pub temperature: f32,
}

pub struct AltimeterMessage {
    pub pressure: f64,
    pub temperature: f64,
}

pub enum GpsMessage {
    GGA {
        fix_time: NaiveTime,
        fix_type: FixType,
        latitude: f64,
        longitude: f64,
        /// MSL Altitude in meters
        altitude: f32,
        num_of_fix_satellites: u8,
        hdop: f32,
        geoid_separation: f32,
    },
    RMC {
        fix_time: NaiveTime,
        fix_date: NaiveDate,
        fix_type: FixType,
        latitude: f64,
        longitude: f64,
        speed_over_ground: f32,
        course_over_ground: f32,
    },
}
