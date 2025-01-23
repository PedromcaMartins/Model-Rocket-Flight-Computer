#![no_std]
#![no_main]

mod io_mapping;

#[allow(unused_imports)]
use defmt::*;

use embassy_stm32::{i2c::I2c, sdmmc::{DataBlock, Sdmmc}, time::Hertz, Config};
use embassy_time::{Delay, Instant, Timer};
use io_mapping::{Bno055I2cMode, IOMapping, SdCard, SdCardDma};
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;

use bno055::{BNO055OperationMode, Bno055};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL168,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 168 / 2 = 168Mhz.
            divq: Some(PllQDiv::DIV7), // 8mhz / 4 * 168 / 7 = 48Mhz.
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
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
    let mut imu = Bno055::new(i2c);

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
async fn sd_card(mut sd_card: Sdmmc<'static, SdCard, SdCardDma>) {
    // Should print 400kHz for initialization
    info!("Configured clock: {}", sd_card.clock().0);

    let mut err = None;
    loop {
        match sd_card.init_card(Hertz::mhz(2)).await {
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

    // Arbitrary block index
    let block_idx = 16;

    // SDMMC uses `DataBlock` instead of `&[u8]` to ensure 4 byte alignment required by the hardware.
    let mut block = DataBlock([0u8; 512]);

    sd_card.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);

    info!("Filling block with 0x55");
    block.fill(0x55);
    sd_card.write_block(block_idx, &block).await.unwrap();
    info!("Write done");

    sd_card.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);

    info!("Filling block with 0xAA");
    block.fill(0xAA);
    sd_card.write_block(block_idx, &block).await.unwrap();
    info!("Write done");

    sd_card.read_block(block_idx, &mut block).await.unwrap();
    info!("Read: {=[u8]:X}...{=[u8]:X}", block[..8], block[512 - 8..]);
}
