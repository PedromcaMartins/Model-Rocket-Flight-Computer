#![no_std]
#![allow(unused_imports)]
#![deny(unsafe_code)]

use core::ops::Deref;

use chrono::{DateTime, NaiveTime, Utc};
use nalgebra::{UnitQuaternion, Vector3};
use nmea::sentences::FixType;
use uom::si::quantities::{Acceleration, Angle, AngularVelocity, Length, MagneticFluxDensity, Pressure, ThermodynamicTemperature, Time};

use postcard_schema::{Schema, schema};
use postcard_rpc::{endpoint, topic};
use serde::{Deserialize, Serialize};

pub type UID = [u8; 12];

endpoint!(PingEndpoint, u32, u32, "ping");
endpoint!(GetUniqueIdEndpoint, (), UID, "unique_id/get");

topic!(AltimeterTopic, AltimeterMessage, "altimeter/data");
topic!(GpsTopic, GpsMessage, "gps/data");
topic!(ImuTopic, ImuMessage, "imu/data");

#[derive(Serialize, Deserialize, Schema, Debug)]
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

#[derive(Serialize, Deserialize, Schema, Debug)]
pub struct GpsMessage {
    /// Timestamp
    pub fix_time: DateTime<Utc>,
    /// Type of GPS Fix
    pub fix_type: FixTypeWraper,
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

#[derive(Serialize, Deserialize, Schema, Debug)]
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



#[derive(Serialize, Deserialize, Debug)]
pub struct FixTypeWraper(FixType);

impl Schema for FixTypeWraper {
    const SCHEMA: &'static schema::NamedType = &schema::NamedType {
        name: "FixType",
        ty: &schema::DataModelType::Enum(&[
            &schema::NamedVariant {
                name: "Invalid",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Gps",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "DGps",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Pps",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Rtk",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "FloatRtk",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Estimated",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Manual",
                ty: &schema::DataModelVariant::UnitVariant,
            },
            &schema::NamedVariant {
                name: "Simulation",
                ty: &schema::DataModelVariant::UnitVariant,
            },
        ]),
    };
}

impl From<FixType> for FixTypeWraper {
    fn from(value: FixType) -> Self {
        Self(value)
    }
}

impl FixTypeWraper {
    pub fn into_inner(self) -> FixType {
        self.0
    }
}
