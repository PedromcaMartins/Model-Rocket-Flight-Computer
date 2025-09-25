use core::fmt::Debug;

use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use embassy_time::Instant;
use embedded_hal::i2c::{I2c, SevenBitAddress};
use telemetry_messages::{AltimeterMessage, Pressure, ThermodynamicTemperature};
use telemetry_messages::uom::si::{pressure::pascal, thermodynamic_temperature::degree_celsius};

use crate::{interfaces::SensorDevice, services::altimeter::altitude_from_pressure};

pub struct Bmp280Device<I, E>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    bmp280: BMP280<I>,
    _error: core::marker::PhantomData<E>,
}

impl<I, E> Bmp280Device<I, E>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    pub fn init(mut bmp280: BMP280<I>) -> Result<Self, E> {
        bmp280.set_config(Config {
            filter: Filter::c16, 
            t_sb: Standby::ms0_5
        })?;

        bmp280.set_control(Control { 
            osrs_t: Oversampling::x1, 
            osrs_p: Oversampling::x4, 
            mode: PowerMode::Normal
        })?;

        Ok(Self {
            bmp280,
            _error: core::marker::PhantomData,
        })
    }
}

impl<I, E> SensorDevice for Bmp280Device<I, E>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    type DataMessage = AltimeterMessage;
    type DeviceError = E;

    #[allow(clippy::cast_possible_truncation)]
    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        let pressure = self.bmp280.pressure()
            .map(|p| p as f32)
            .map(Pressure::new::<pascal>)?;
        let temperature = self.bmp280.temp()
            .map(|t| t as f32)
            .map(ThermodynamicTemperature::new::<degree_celsius>)?;

        let altitude = altitude_from_pressure(pressure);

        Ok(AltimeterMessage {
            altitude,
            pressure,
            temperature, 
            timestamp: Instant::now().as_micros(),
        })
    }
}
