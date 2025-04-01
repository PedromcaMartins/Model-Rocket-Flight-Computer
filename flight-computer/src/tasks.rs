use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use bno055::{BNO055OperationMode, BNO055PowerMode, Bno055};

use defmt::Debug2Format;
use embassy_stm32::{i2c::I2c, sdmmc::Sdmmc, time::Hertz, usart::Uart, mode};
use embassy_time::{Delay, Instant, Timer};
use nmea::{Nmea, SentenceType};

use crate::io_mapping::{Bmp280I2cMode, Bno055I2cMode, SdCard, SdCardDma};


#[embassy_executor::task]
pub async fn imu(i2c: I2c<'static, Bno055I2cMode>) {
    // The sensor has an initial startup time of 400ms - 650ms during which interaction with it will fail
    Timer::at(Instant::from_millis(650)).await;

    let mut delay = Delay;
    let mut imu = Bno055::new(i2c).with_alternative_address();

    imu.init(&mut delay).unwrap();

    // Enable 9-degrees-of-freedom sensor fusion mode with fast magnetometer calibration
    imu.set_mode(BNO055OperationMode::NDOF, &mut delay).unwrap();

    // Set power mode to normal
    imu.set_power_mode(BNO055PowerMode::NORMAL).unwrap();

    // Enable usage of external crystal
    imu.set_external_crystal(true, &mut delay).unwrap();

    loop {
        match imu.euler_angles() {
            Ok(val) => {
                defmt::info!("IMU angles: {:?}", Debug2Format(&val));
            }
            Err(e) => {
                defmt::error!("{:?}", e);
            }
        }

        match imu.quaternion() {
            Ok(val) => {
                defmt::info!("IMU quaternion: {:?}", Debug2Format(&val));
            }
            Err(e) => {
                defmt::error!("{:?}", e);
            }
        }

        match imu.linear_acceleration() {
            Ok(val) => {
                defmt::info!("IMU linear acceleration: {:?}", Debug2Format(&val));
            }
            Err(e) => {
                defmt::error!("{:?}", e);
            }
        }

        match imu.gravity() {
            Ok(val) => {
                defmt::info!("IMU gravity: {:?}", Debug2Format(&val));
            }
            Err(e) => {
                defmt::error!("{:?}", e);
            }
        }

        match imu.accel_data() {
            Ok(val) => {
                defmt::info!("IMU acceleration: {:?}", Debug2Format(&val));
            }
            Err(e) => {
                defmt::error!("{:?}", e);
            }
        }

        match imu.gyro_data() {
            Ok(val) => {
                defmt::info!("IMU gyro: {:?}", Debug2Format(&val));
            }
            Err(e) => {
                defmt::error!("{:?}", e);
            }
        }

        match imu.mag_data() {
            Ok(val) => {
                defmt::info!("IMU mag: {:?}", Debug2Format(&val));
            }
            Err(e) => {
                defmt::error!("{:?}", e);
            }
        }

        match imu.temperature() {
            Ok(val) => {
                defmt::info!("IMU temperature: {:?}", val);
            }
            Err(e) => {
                defmt::error!("{:?}", e);
            }
        }

        Timer::after_millis(500).await;
    }
}

mod altitude {
    use micromath::F32Ext;

    /// Calculates altitude (in meters) from pressure (Pa)
    /// Uses the standard barometric formula.
    ///
    /// - `pressure`: The measured atmospheric pressure in Pascals.
    /// - `temperature`: The measured temperature in Kelvin.
    ///
    /// Returns altitude in meters.
    pub fn pressure_to_altitude(pressure: f32) -> f32 {
        const P0: f32 = 101325.0; // Sea level standard atmospheric pressure (Pa)
        const MAGIC_NUMBER: f32 = 0.1902632; // Magic number for altitude calculation
        const MAGIC_NUMBER_2: f32 = 44330.77; // Magic number for altitude calculation

        let ratio = pressure / P0;

        MAGIC_NUMBER_2 * (1.0 - ratio.powf(MAGIC_NUMBER))
    }
}

#[embassy_executor::task]
pub async fn altimeter(i2c: I2c<'static, Bmp280I2cMode>) {
    let mut altimeter = BMP280::new(i2c).unwrap();

    altimeter.set_config(Config {
        filter: Filter::c16, 
        t_sb: Standby::ms0_5
    });

    altimeter.set_control(Control { 
        osrs_t: Oversampling::x1, 
        osrs_p: Oversampling::x4, 
        mode: PowerMode::Normal
    });

    loop {
        let pressure = altimeter.pressure();
        let temperature = altimeter.temp() as f32;

        let altitude = altitude::pressure_to_altitude(pressure as f32);

        defmt::info!("Pressure: {:?} Pa, Temperature: {:?} °C, Altitude: {:?} m", pressure, temperature, altitude);
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
pub async fn sd_card(mut sd_card: Sdmmc<'static, SdCard, SdCardDma>) {
    // Should print 400kHz for initialization
    defmt::info!("Configured clock: {}", sd_card.clock().0);

    let mut err = None;
    loop {
        match sd_card.init_card(Hertz::mhz(25)).await {
            Ok(_) => break,
            Err(e) => {
                if err != Some(e) {
                    defmt::info!("waiting for card error, retrying: {:?}", e);
                    err = Some(e);
                }
            }
        }
    }

    let card = defmt::unwrap!(sd_card.card());

    defmt::info!("Card: {:#?}", Debug2Format(card));
    defmt::info!("Clock: {}", sd_card.clock());
}

#[embassy_executor::task]
pub async fn gps(mut uart: Uart<'static, mode::Async>) {
    let mut buf = [0; nmea::SENTENCE_MAX_LEN];
    let mut nmea = Nmea::create_for_navigation(&[SentenceType::GGA]).unwrap();

    loop {
        let len = uart.read_until_idle(&mut buf).await.unwrap();
        let message = core::str::from_utf8(&buf[..len]).unwrap();

        match nmea.parse(message) {
            Ok(_) => {
                defmt::info!("GPS: {:?}", nmea);
            }
            Err(e) => {
                defmt::error!("{:?}", defmt::Debug2Format(&e));
            }
        }
    }
}
