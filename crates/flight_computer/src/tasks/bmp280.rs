use core::fmt::Debug;

use defmt_or_log::info;
use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use embassy_time::Timer;
use embedded_hal::i2c::{I2c, SevenBitAddress};
use uom::si::{f32::{Length, ThermodynamicTemperature}, f64::Pressure, pressure::pascal, thermodynamic_temperature::degree_celsius, length::meter};

#[inline]
pub async fn bmp280_task<I, E>(mut bmp280: BMP280<I>)
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    bmp280.set_config(Config {
        filter: Filter::c16, 
        t_sb: Standby::ms0_5
    }).unwrap();

    bmp280.set_control(Control { 
        osrs_t: Oversampling::x1, 
        osrs_p: Oversampling::x4, 
        mode: PowerMode::Normal
    }).unwrap();

    loop {
        if let (Ok(pressure), Ok(temperature)) = (bmp280.pressure(), bmp280.temp()) {
            let pressure = Pressure::new::<pascal>(pressure);
            #[allow(clippy::cast_possible_truncation)]
            let temperature = ThermodynamicTemperature::new::<degree_celsius>(temperature as f32);
            let altitude = altitude_from_pressure(pressure);
    
            info!("Pressure: {:?} Pa, Temperature: {:?} °C, Altitude: {:?} m", 
                pressure.get::<pascal>(), 
                temperature.get::<degree_celsius>(),
                altitude.get::<meter>(),
            );
        }

        Timer::after_millis(100).await;
    }
}

fn altitude_from_pressure(pressure: Pressure) -> Length {
    #[allow(unused_imports)]
    use uom::num_traits::Float;

    #[allow(clippy::cast_possible_truncation)]
    let pressure = pressure.get::<pascal>() as f32;
    let p0 = 101_325.0_f32; // ISA sea level standard pressure in pascal
    let exponent = 0.190_284_f32;
    let scale = 44_330.0_f32;

    let pressure_ratio = pressure / p0;
    let altitude_m = scale * (1.0 - pressure_ratio.powf(exponent));

    Length::new::<meter>(altitude_m)
}
