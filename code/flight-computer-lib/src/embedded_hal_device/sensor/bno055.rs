use core::fmt::Debug;

use bno055::{BNO055OperationMode, BNO055PowerMode, Bno055};
use embassy_time::{Delay, Instant, Timer};
use embedded_hal::i2c::{I2c, SevenBitAddress};
use telemetry_messages::{nalgebra::{Quaternion, Vector3, Vector4}, EulerAngles, ImuMessage};
use uom::si::{acceleration::meter_per_second_squared, angle::degree, angular_velocity::degree_per_second, magnetic_flux_density::microtesla, quantities::{Acceleration, Angle, AngularVelocity, MagneticFluxDensity, ThermodynamicTemperature}, thermodynamic_temperature::degree_celsius};

use crate::model::sensor_device::SensorDevice;

pub struct Bno055Device<I, E>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    bno055: Bno055<I>,
    _error: core::marker::PhantomData<E>,
}

impl<I, E> Bno055Device<I, E>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    pub async fn init(mut bno055: Bno055<I>) -> Result<Self, bno055::Error<E>> {
        // The sensor has an initial startup time of 400ms - 650ms during which interaction with it will fail
        Timer::at(Instant::from_millis(650)).await;
        let mut delay = Delay;

        bno055.init(&mut delay)?;

        // Enable 9-degrees-of-freedom sensor fusion mode with fast magnetometer calibration
        bno055.set_mode(BNO055OperationMode::NDOF, &mut delay)?;

        // Set power mode to normal
        bno055.set_power_mode(BNO055PowerMode::NORMAL)?;

        // Enable usage of external crystal
        bno055.set_external_crystal(true, &mut delay)?;

        Ok(Self {
            bno055,
            _error: core::marker::PhantomData,
        })
    }
}

impl<I, E> SensorDevice for Bno055Device<I, E>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    type DataMessage = ImuMessage;
    type DeviceError = bno055::Error<E>;

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        let euler_angles = self.bno055.euler_angles()?;
        let quaternion = self.bno055.quaternion()?;
        let linear_acceleration = self.bno055.linear_acceleration()?;
        let gravity = self.bno055.gravity()?;
        let acceleration = self.bno055.accel_data()?;
        let gyro = self.bno055.gyro_data()?;
        let mag = self.bno055.mag_data()?;
        let temperature = self.bno055.temperature()?;

        let euler_angles = EulerAngles {
            roll: Angle::new::<degree>(euler_angles.c),
            pitch: Angle::new::<degree>(euler_angles.a),
            yaw: Angle::new::<degree>(euler_angles.b),
        };

        let quaternion = Quaternion::from_vector(
            Vector4::new(
                quaternion.v.x,
                quaternion.v.y,
                quaternion.v.z,
                quaternion.s, 
            )
        );

        let linear_acceleration = Vector3::new(
            Acceleration::new::<meter_per_second_squared>(linear_acceleration.x), 
            Acceleration::new::<meter_per_second_squared>(linear_acceleration.y),
            Acceleration::new::<meter_per_second_squared>(linear_acceleration.z) 
        );
        let gravity = Vector3::new(
            Acceleration::new::<meter_per_second_squared>(gravity.x), 
            Acceleration::new::<meter_per_second_squared>(gravity.y),
            Acceleration::new::<meter_per_second_squared>(gravity.z) 
        );
        let acceleration = Vector3::new(
            Acceleration::new::<meter_per_second_squared>(acceleration.x), 
            Acceleration::new::<meter_per_second_squared>(acceleration.y),
            Acceleration::new::<meter_per_second_squared>(acceleration.z) 
        );
        let gyro = Vector3::new(
            AngularVelocity::new::<degree_per_second>(gyro.x), 
            AngularVelocity::new::<degree_per_second>(gyro.y),
            AngularVelocity::new::<degree_per_second>(gyro.z) 
        );
        let mag = Vector3::new(
            MagneticFluxDensity::new::<microtesla>(mag.x), 
            MagneticFluxDensity::new::<microtesla>(mag.y),
            MagneticFluxDensity::new::<microtesla>(mag.z) 
        );
        let temperature = 
            ThermodynamicTemperature::new::<degree_celsius>(temperature.into());

        Ok(ImuMessage {
            euler_angles,
            quaternion,
            linear_acceleration,
            gravity,
            acceleration,
            gyro,
            mag,
            temperature,
            timestamp: Instant::now().as_micros(),
        })
    }
}
