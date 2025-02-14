#![no_std]
#![no_main]

mod io_mapping;

use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
#[allow(unused_imports)]
use defmt::*;

use embassy_stm32::{i2c::I2c, sdmmc::{DataBlock, Sdmmc}, time::Hertz};
use embassy_time::{Delay, Instant, Timer};
use io_mapping::{Bmp280I2cMode, Bno055I2cMode, IOMapping, SdCard, SdCardDma};
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;

use bno055::{BNO055OperationMode, Bno055};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        use embassy_stm32::rcc::mux::*;
        config.rcc.hsi = true;
        config.rcc.hse = None;
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.pll_src = PllSource::HSI;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV8,
            mul: PllMul::MUL100,
            divp: Some(PllPDiv::DIV2), // 16mhz / 8 * 96 / 2 = 96Mhz.
            divq: Some(PllQDiv::DIV4), // 16mhz / 8 * 96 / 4 = 48Mhz.
            divr: None,
        });
        config.rcc.plli2s = Some(Pll { 
            prediv: PllPreDiv::DIV16, 
            mul: PllMul::MUL192, 
            divp: None, 
            divq: Some(PllQDiv::DIV2), // 16mhz / 16 * 192 / 2 = 96Mhz.
            divr: None, 
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;

        config.rcc.mux.sdiosel = Sdiosel::CLK48;
    }
    let p = embassy_stm32::init(config);
    let io_mapping = IOMapping::init(p);

    unwrap!(spawner.spawn(imu(io_mapping.bno055_i2c)));
    unwrap!(spawner.spawn(sd_card(io_mapping.sd_card)));
}

#[embassy_executor::task]
async fn imu(i2c: I2c<'static, Bno055I2cMode>) {
    // The sensor has an initial startup time of 400ms - 650ms during which interaction with it will fail
    Timer::at(Instant::from_millis(650)).await;

    let mut delay = Delay;
    let mut imu = Bno055::new(i2c).with_alternative_address();

    imu.init(&mut delay).unwrap();

    // Enable 9-degrees-of-freedom sensor fusion mode with fast magnetometer calibration
    imu.set_mode(BNO055OperationMode::NDOF, &mut delay).unwrap();

    let mut euler_angles;

    loop {
        match imu.euler_angles() {
            Ok(val) => {
                euler_angles = val;
                info!("IMU angles: ({:?}, {:?}, {:?})", euler_angles.a, euler_angles.b, euler_angles.c);
                Timer::after_millis(500).await;
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
}

#[embassy_executor::task]
async fn altimeter(i2c: I2c<'static, Bmp280I2cMode>) {
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

    let pressure = altimeter.pressure();
    let temperature = altimeter.temp();

    loop {
        info!("Pressure: {:?} Pa, Temperature: {:?} °C", pressure, temperature);
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn sd_card(mut sd_card: Sdmmc<'static, SdCard, SdCardDma>) {
    // Should print 400kHz for initialization
    info!("Configured clock: {}", sd_card.clock().0);

    let mut err = None;
    loop {
        match sd_card.init_card(Hertz::mhz(25)).await {
            Ok(_) => break,
            Err(e) => {
                if err != Some(e) {
                    info!("waiting for card error, retrying: {:?}", e);
                    err = Some(e);
                }
            }
        }
    }

    let card = unwrap!(sd_card.card());

    info!("Card: {:#?}", Debug2Format(card));
    info!("Clock: {}", sd_card.clock());
}
