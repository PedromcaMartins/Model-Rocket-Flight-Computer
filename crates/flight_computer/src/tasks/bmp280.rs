use core::fmt::Debug;

use defmt::info;
use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use embassy_time::Timer;
use embedded_hal::i2c::{I2c, SevenBitAddress};

#[inline]
pub async fn bmp280_task<I, E>(mut bmp280: BMP280<I>)
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    bmp280.set_config(Config {
        filter: Filter::c16, 
        t_sb: Standby::ms0_5
    });

    bmp280.set_control(Control { 
        osrs_t: Oversampling::x1, 
        osrs_p: Oversampling::x4, 
        mode: PowerMode::Normal
    });

    loop {
        let pressure = bmp280.pressure();
        #[allow(clippy::cast_possible_truncation)]
        let temperature = bmp280.temp() as f32;

        info!("Pressure: {:?} Pa, Temperature: {:?} °C", pressure, temperature);

        Timer::after_millis(100).await;
    }
}
