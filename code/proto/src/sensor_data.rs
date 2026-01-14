use crate::{Deserialize, Serialize, Schema, FixTypeWrapper, NaiveTimeWrapper};

pub use nmea;
pub use nalgebra::Vector3;
pub use uom::si::f32::{Acceleration, Angle, AngularVelocity, Length, MagneticFluxDensity, Pressure, Time, ThermodynamicTemperature, Velocity};

/* ------------------------------ Type Aliases ------------------------------ */

pub type Altitude = Length;

/* ------------------------------ Altimeter Data ---------------------------- */

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq)]
pub struct AltimeterData {
    /// Pressure in Pascal.
    pub pressure: Pressure,
    /// Altitude in meters.
    pub altitude: Altitude,
    /// Temperature in Celsius degrees.
    pub temperature: ThermodynamicTemperature,
}

/* --------------------------------- Gps Data ------------------------------- */

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq)]
pub struct GpsCoordinates {
    /// Latitude in degrees.
    pub latitude: f32,
    /// Longitude in degrees.
    pub longitude: f32,
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq)]
pub struct GpsData {
    /// Timestamp
    pub fix_time: NaiveTimeWrapper,
    /// Type of GPS Fix
    pub fix_type: FixTypeWrapper,
    /// Coordinates
    pub coordinates: GpsCoordinates,
    /// MSL Altitude in meters
    pub altitude: Altitude,
    /// Number of satellites used for fix.
    pub num_of_fix_satellites: u8,
}

/* --------------------------------- Imu Data ------------------------------- */

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq)]
pub struct ImuData {
    /// Acceleration vector in m/s^2 units.
    pub acceleration: Vector3<Acceleration>,
    /// Gyroscope vector in deg/s units.
    pub gyro: Vector3<AngularVelocity>,
    /// Magnetometer vector in uT units.
    pub mag: Vector3<MagneticFluxDensity>,
    /// Temperature of the chip in Celsius degrees.
    pub temperature: ThermodynamicTemperature,
}
