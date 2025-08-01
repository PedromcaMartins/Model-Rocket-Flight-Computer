use core::fmt::Debug;

use defmt_or_log::{info, error, Debug2Format};
use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use embassy_time::{Instant, Timer};
use embedded_hal::i2c::{I2c, SevenBitAddress};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{AltimeterMessage, AltimeterTopic};
use uom::si::{length::meter, pressure::pascal, quantities::{Length, Pressure, ThermodynamicTemperature, Time}, thermodynamic_temperature::degree_celsius, time::microsecond};

#[inline]
pub async fn bmp280_task<I, E, Tx>(
    bmp280: BMP280<I>,
    sender: PostcardSender<Tx>,
) -> !
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
    Tx: WireTx,
{
    let mut parser = Bmp280Parser::init(bmp280).unwrap();
    let mut seq = 0_u32;

    loop {
        if let Ok(msg) = parser.parse_new_message() {
            info!("Altitude Message {:#?}", Debug2Format(&msg));

            let _ = sender.publish::<AltimeterTopic>(VarSeq::Seq4(seq), &msg).await;
            seq = seq.wrapping_add(1);
        } else {
            error!("Failed to read BMP280");
        }

        Timer::after_millis(100).await;
    }
}
struct Bmp280Parser<I, E>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    bmp280: BMP280<I>,
    _error: core::marker::PhantomData<E>,
}

impl<I, E> Bmp280Parser<I, E>
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

    pub fn parse_new_message(&mut self) -> Result<AltimeterMessage, E> {
        let pressure = self.bmp280.pressure()
            .map(Pressure::new::<pascal>)?;
        let temperature = self.bmp280.temp()
            .map(ThermodynamicTemperature::new::<degree_celsius>)?;

        let altitude = Self::altitude_from_pressure(pressure);

        Ok(AltimeterMessage {
            altitude,
            pressure,
            temperature, 
            timestamp: Time::new::<microsecond>(Instant::now().as_micros()),
        })
    }

    fn altitude_from_pressure(pressure: Pressure<f64>) -> Length<f64> {
        #[allow(unused_imports)]
        use uom::num_traits::Float;

        #[allow(clippy::cast_possible_truncation)]
        let pressure = pressure.get::<pascal>() as f32;
        let p0 = 101_325.0_f32; // ISA sea level standard pressure in pascal
        let exponent = 0.190_284_f32;
        let scale = 44_330.0_f32;

        let pressure_ratio = pressure / p0;
        let altitude_m = scale * (1.0 - pressure_ratio.powf(exponent));

        Length::new::<meter>(altitude_m.into())
    }
}
