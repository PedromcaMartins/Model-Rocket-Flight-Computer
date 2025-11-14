#![no_std]
#![deny(unsafe_code)]
#![deny(unused_must_use)]

use postcard_schema::{Schema, schema};
use postcard_rpc::{endpoints, topics, TopicDirection};
use defmt::Format;

pub use nalgebra;
pub use nmea;
pub use uom;
pub use serde::{Deserialize, Serialize};
pub use nalgebra::Vector3;
pub use uom::si::f32::{Acceleration, Angle, AngularVelocity, Length, MagneticFluxDensity, Pressure, Time, ThermodynamicTemperature, Velocity};

mod newtypes;
pub use newtypes::*;

mod log_data_type;
pub use log_data_type::*;

/* ------------------- Postcard RPC Endpoint Configuration ------------------ */

endpoints! {
    list = ENDPOINT_LIST;
    omit_std = true;
    | EndpointTy                | RequestTy     | ResponseTy            | Path              |
    | ----------                | ---------     | ----------            | ----              |
    | PingEndpoint              | u32           | u32                   | "ping"            |
}

topics! {
    list = TOPICS_IN_LIST;
    direction = TopicDirection::ToServer;
    | TopicTy                   | MessageTy     | Path              |
    | -------                   | ---------     | ----              |
}

topics! {
    list = TOPICS_OUT_LIST;
    direction = TopicDirection::ToClient;
    | TopicTy                   | MessageTy         | Path              | Cfg                           |
    | -------                   | ---------         | ----              | ---                           |
    | AltimeterTopic            | AltimeterMessage  | "altimeter/data"  |                               |
    | GpsTopic                  | GpsMessage        | "gps/data"        |                               |
    | ImuTopic                  | ImuMessage        | "imu/data"        |                               |
}

/* ------------------------------ Type Aliases ------------------------------ */

pub type Altitude = Length;
pub type Quaternion = nalgebra::Quaternion<f32>;

/* ------------------------------ Flight State ------------------------------ */

#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, Format, PartialEq, Eq)]
pub enum FlightState {
    PreArmed,
    Armed,
    RecoveryActivated,
    Touchdown,
}

/* ---------------------------- Altimeter Message --------------------------- */

#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq)]
pub struct AltimeterMessage {
    /// Pressure in Pascal.
    pub pressure: Pressure,
    /// Altitude in meters.
    pub altitude: Altitude,
    /// Temperature in Celsius degrees.
    pub temperature: ThermodynamicTemperature,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

impl LogMessage for AltimeterMessage {
    const KIND: LogDataType = LogDataType::Altimeter;
}

/* ------------------------------- Gps Message ------------------------------ */

#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq)]
pub struct GpsCoordinates {
    /// Latitude in degrees.
    pub latitude: f32,
    /// Longitude in degrees.
    pub longitude: f32,
}

#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq)]
pub struct GpsMessage {
    /// Timestamp
    pub fix_time: NaiveTimeWrapper,
    /// Type of GPS Fix
    pub fix_type: FixTypeWrapper,
    /// Gps Coordinates
    pub coordinates: GpsCoordinates,
    /// MSL Altitude in meters
    pub altitude: Altitude,
    /// Number of satellites used for fix.
    pub num_of_fix_satellites: u8,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

impl LogMessage for GpsMessage {
    const KIND: LogDataType = LogDataType::Gps;
}

/* ------------------------------- Imu Message ------------------------------ */

#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq)]
pub struct EulerAngles {
    pub roll:  Angle,
    pub pitch: Angle,
    pub yaw:   Angle,
}

#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq)]
pub struct ImuMessage {
    /// Euler angles representation of heading in degrees.
    /// Euler angles is represented as (`roll`, `pitch`, `yaw/heading`).
    pub euler_angles: EulerAngles,
    /// Standard quaternion represented by the scalar and vector parts. Corresponds to a right-handed rotation matrix.
    /// Quaternion is represented as (x, y, z, s).
    ///
    /// where:
    /// x, y, z: Vector part of a quaternion;
    /// s: Scalar part of a quaternion.
    pub quaternion: Quaternion,
    /// Linear acceleration vector in m/s^2 units.
    pub linear_acceleration: Vector3<Acceleration>,
    /// Gravity vector in m/s^2 units.
    pub gravity: Vector3<Acceleration>,
    /// Acceleration vector in m/s^2 units.
    pub acceleration: Vector3<Acceleration>,
    /// Gyroscope vector in deg/s units.
    pub gyro: Vector3<AngularVelocity>,
    /// Magnetometer vector in uT units.
    pub mag: Vector3<MagneticFluxDensity>,
    /// Temperature of the chip in Celsius degrees.
    pub temperature: ThermodynamicTemperature,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

impl LogMessage for ImuMessage {
    const KIND: LogDataType = LogDataType::Imu;
}
