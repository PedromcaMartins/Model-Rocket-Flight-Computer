use core::fmt::Debug;

use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use embassy_time::Instant;
use embedded_hal::i2c::{I2c, SevenBitAddress};
use telemetry_messages::AltimeterMessage;
use uom::si::{pressure::pascal, quantities::{Pressure, ThermodynamicTemperature, Time}, thermodynamic_temperature::degree_celsius, time::microsecond};

use crate::{device::sensor::SensorDevice, model::altimeter::altitude_from_pressure};

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

    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        let pressure = self.bmp280.pressure()
            .map(Pressure::new::<pascal>)?;
        let temperature = self.bmp280.temp()
            .map(ThermodynamicTemperature::new::<degree_celsius>)?;

        let altitude = altitude_from_pressure(pressure);

        Ok(AltimeterMessage {
            altitude,
            pressure,
            temperature, 
            timestamp: Time::new::<microsecond>(Instant::now().as_micros() as f64),
        })
    }
}
