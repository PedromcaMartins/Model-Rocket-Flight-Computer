use nmea::sentences::FixType;
use proto::{uom, sensor_data::{Acceleration, AltimeterData, Altitude, Angle, AngularVelocity, GpsData, ImuData, Pressure, ThermodynamicTemperature, Vector3}};
use rstest::fixture;

#[fixture]
pub fn random_altimeter_data() -> AltimeterData {
    AltimeterData {
        pressure: Pressure::new::<uom::si::pressure::pascal>(rand::random()),
        altitude: Altitude::new::<uom::si::length::meter>(rand::random()),
        temperature: ThermodynamicTemperature::new::<uom::si::thermodynamic_temperature::degree_celsius>(rand::random()),
    }
}

#[fixture]
pub fn random_gps_data() -> GpsData {
    GpsData {
        fix_type: FixType::Invalid.into(),
        fix_time: chrono::NaiveTime::from_hms_opt(
            rand::random_range(0..24), 
            rand::random_range(0..60), 
            rand::random_range(0..60),
        ).expect("Failed to create NaiveTime").into(),
        latitude: Angle::new::<uom::si::angle::degree>(rand::random()),
        longitude: Angle::new::<uom::si::angle::degree>(rand::random()),
        altitude: Altitude::new::<uom::si::length::meter>(rand::random()),
        num_of_fix_satellites: rand::random_range(0..20),
    }
}

#[fixture]
pub fn random_imu_data() -> ImuData {
    ImuData {
        acceleration: Vector3::new(
            Acceleration::new::<uom::si::acceleration::meter_per_second_squared>(rand::random()),
            Acceleration::new::<uom::si::acceleration::meter_per_second_squared>(rand::random()),
            Acceleration::new::<uom::si::acceleration::meter_per_second_squared>(rand::random()),
        ),
        gyro: Vector3::new(
            AngularVelocity::new::<uom::si::angular_velocity::degree_per_second>(rand::random()),
            AngularVelocity::new::<uom::si::angular_velocity::degree_per_second>(rand::random()),
            AngularVelocity::new::<uom::si::angular_velocity::degree_per_second>(rand::random()),
        ),
        mag: Vector3::new(
            proto::sensor_data::MagneticFluxDensity::new::<uom::si::magnetic_flux_density::microtesla>(rand::random()),
            proto::sensor_data::MagneticFluxDensity::new::<uom::si::magnetic_flux_density::microtesla>(rand::random()),
            proto::sensor_data::MagneticFluxDensity::new::<uom::si::magnetic_flux_density::microtesla>(rand::random()),
        ),
        temperature: ThermodynamicTemperature::new::<uom::si::thermodynamic_temperature::degree_celsius>(rand::random()),
    }
}
