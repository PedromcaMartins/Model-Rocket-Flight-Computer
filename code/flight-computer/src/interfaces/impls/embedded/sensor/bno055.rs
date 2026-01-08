use core::fmt::Debug;

use bno055::{BNO055OperationMode, BNO055PowerMode, Bno055};
use embassy_time::{Delay, Instant, Timer};
use embedded_hal::i2c::{I2c, SevenBitAddress};
use proto::sensor_data::{Vector3, ImuData};
use proto::uom::si::{acceleration::meter_per_second_squared, angular_velocity::degree_per_second, magnetic_flux_density::microtesla, thermodynamic_temperature::degree_celsius};
use proto::sensor_data::{Acceleration, AngularVelocity, MagneticFluxDensity, ThermodynamicTemperature};

use crate::interfaces::SensorDevice;

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
    type Data = ImuData;
    type Error = bno055::Error<E>;

    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error> {
        let acceleration = self.bno055.accel_data()?;
        let gyro = self.bno055.gyro_data()?;
        let mag = self.bno055.mag_data()?;
        let temperature = self.bno055.temperature()?;

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

        Ok(ImuData {
            acceleration,
            gyro,
            mag,
            temperature,
        })
    }
}
