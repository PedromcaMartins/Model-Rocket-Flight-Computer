use core::fmt::Debug;

use defmt::{info, error};
use bno055::{BNO055OperationMode, BNO055PowerMode, Bno055};
use defmt::Debug2Format;
use embassy_time::{Delay, Instant, Timer};
use embedded_hal::i2c::{I2c, SevenBitAddress};

#[inline]
pub async fn bno055_task<I, E>(mut bno055: Bno055<I>)
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    // The sensor has an initial startup time of 400ms - 650ms during which interaction with it will fail
    Timer::at(Instant::from_millis(650)).await;

    let mut delay = Delay;

    bno055.init(&mut delay).unwrap();

    // Enable 9-degrees-of-freedom sensor fusion mode with fast magnetometer calibration
    bno055.set_mode(BNO055OperationMode::NDOF, &mut delay).unwrap();

    // Set power mode to normal
    bno055.set_power_mode(BNO055PowerMode::NORMAL).unwrap();

    // Enable usage of external crystal
    bno055.set_external_crystal(true, &mut delay).unwrap();

    loop {
        match bno055.euler_angles() {
            Ok(val) => info!("bno055 angles: {:?}", Debug2Format(&val)),
            Err(e) => error!("{:?}", Debug2Format(&e)),
        }

        match bno055.quaternion() {
            Ok(val) => info!("bno055 quaternion: {:?}", Debug2Format(&val)),
            Err(e) => error!("{:?}", Debug2Format(&e)),
        }

        match bno055.linear_acceleration() {
            Ok(val) => info!("bno055 linear acceleration: {:?}", Debug2Format(&val)),
            Err(e) => error!("{:?}", Debug2Format(&e)),
        }

        match bno055.gravity() {
            Ok(val) => info!("bno055 gravity: {:?}", Debug2Format(&val)),
            Err(e) => error!("{:?}", Debug2Format(&e)),
        }

        match bno055.accel_data() {
            Ok(val) => info!("bno055 acceleration: {:?}", Debug2Format(&val)),
            Err(e) => error!("{:?}", Debug2Format(&e)),
        }

        match bno055.gyro_data() {
            Ok(val) => info!("bno055 gyro: {:?}", Debug2Format(&val)),
            Err(e) => error!("{:?}", Debug2Format(&e)),
        }

        match bno055.mag_data() {
            Ok(val) => info!("bno055 mag: {:?}", Debug2Format(&val)),
            Err(e) => error!("{:?}", Debug2Format(&e)),
        }

        match bno055.temperature() {
            Ok(val) => info!("bno055 temperature: {:?}", val),
            Err(e) => error!("{:?}", Debug2Format(&e)),
        }

        Timer::after_millis(100).await;
    }
}
