#![no_std]
#![no_main]
#![deny(unsafe_code)]

mod io_mapping;
use io_mapping::IOMapping;

use crate::io_mapping::{Bmp280Port, Bno055Port, DebugUartPort, SdCardDetectPort, SdCardPort, SdCardStatusLedPort, UbloxNeo7mPort};

use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;

use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use bno055::{BNO055OperationMode, BNO055PowerMode, Bno055};

use defmt::Debug2Format;
use embassy_stm32::time::Hertz;
use embassy_time::{Delay, Duration, Instant, Ticker, Timer};
use nmea::{Nmea, SentenceType};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(get_config());
    let io_mapping = IOMapping::init(p);
    let IOMapping {
        bno055_port,
        bmp280_port,
        sd_card_port,
        sd_card_detect_port,
        sd_card_status_led_port,
        debug_uart_port,
        ublox_neo_7m_port,
    } = io_mapping;

    defmt::info!("{:#?}", embassy_stm32::uid::uid());

    spawner.must_spawn(bno055_task(bno055_port));
    spawner.must_spawn(bmp280_task(bmp280_port));
    spawner.must_spawn(sd_card_task(sd_card_port, sd_card_detect_port, sd_card_status_led_port));
    spawner.must_spawn(gps(ublox_neo_7m_port));
    spawner.must_spawn(debug_uart(debug_uart_port));
}

fn get_config() -> embassy_stm32::Config {
    use embassy_stm32::rcc::*;
    use embassy_stm32::rcc::mux::*;

    let mut config = embassy_stm32::Config::default();
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

    config
}

#[embassy_executor::task]
async fn bno055_task(bno055_port: Bno055Port) {
    // The sensor has an initial startup time of 400ms - 650ms during which interaction with it will fail
    Timer::at(Instant::from_millis(650)).await;

    let mut delay = Delay;
    let mut bno055 = Bno055::new(bno055_port).with_alternative_address();

    bno055.init(&mut delay).unwrap();

    // Enable 9-degrees-of-freedom sensor fusion mode with fast magnetometer calibration
    bno055.set_mode(BNO055OperationMode::NDOF, &mut delay).unwrap();

    // Set power mode to normal
    bno055.set_power_mode(BNO055PowerMode::NORMAL).unwrap();

    // Enable usage of external crystal
    bno055.set_external_crystal(true, &mut delay).unwrap();

    loop {
        match bno055.euler_angles() {
            Ok(val) => defmt::info!("bno055 angles: {:?}", Debug2Format(&val)),
            Err(e) => defmt::error!("{:?}", e),
        }

        match bno055.quaternion() {
            Ok(val) => defmt::info!("bno055 quaternion: {:?}", Debug2Format(&val)),
            Err(e) => defmt::error!("{:?}", e),
        }

        match bno055.linear_acceleration() {
            Ok(val) => defmt::info!("bno055 linear acceleration: {:?}", Debug2Format(&val)),
            Err(e) => defmt::error!("{:?}", e),
        }

        match bno055.gravity() {
            Ok(val) => defmt::info!("bno055 gravity: {:?}", Debug2Format(&val)),
            Err(e) => defmt::error!("{:?}", e),
        }

        match bno055.accel_data() {
            Ok(val) => defmt::info!("bno055 acceleration: {:?}", Debug2Format(&val)),
            Err(e) => defmt::error!("{:?}", e),
        }

        match bno055.gyro_data() {
            Ok(val) => defmt::info!("bno055 gyro: {:?}", Debug2Format(&val)),
            Err(e) => defmt::error!("{:?}", e),
        }

        match bno055.mag_data() {
            Ok(val) => defmt::info!("bno055 mag: {:?}", Debug2Format(&val)),
            Err(e) => defmt::error!("{:?}", e),
        }

        match bno055.temperature() {
            Ok(val) => defmt::info!("bno055 temperature: {:?}", val),
            Err(e) => defmt::error!("{:?}", e),
        }

        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn bmp280_task(bmp280_port: Bmp280Port) {
    let mut bmp280 = BMP280::new(bmp280_port).unwrap();

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
        let temperature = bmp280.temp() as f32;

        defmt::info!("Pressure: {:?} Pa, Temperature: {:?} °C", pressure, temperature);

        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn sd_card_task(mut sd_card_port: SdCardPort, sd_card_detect_port: SdCardDetectPort, mut sd_card_status_led_port: SdCardStatusLedPort) {
    // Should print 400kHz for initialization
    defmt::info!("Configured clock: {}", sd_card_port.clock().0);

    let mut err = None;
    loop {
        match sd_card_port.init_card(Hertz::mhz(25)).await {
            Ok(_) => break,
            Err(e) => {
                if err != Some(e) {
                    defmt::error!("waiting for card error, retrying: {:?}", e);
                    err = Some(e);
                }
            }
        }
    }

    let sd_card = defmt::unwrap!(sd_card_port.card());

    defmt::info!("Card: {:#?}", Debug2Format(sd_card));
    defmt::info!("Clock: {}", sd_card_port.clock());
    defmt::info!("Sd Card Detect State: {:#?}", sd_card_detect_port.get_level());

    let mut tick = Ticker::every(Duration::from_secs(1));
    loop {
        sd_card_status_led_port.toggle();
        tick.next().await;
    }
}

#[embassy_executor::task]
async fn gps(mut uart: UbloxNeo7mPort) {
    let mut buf = [0; nmea::SENTENCE_MAX_LEN];
    let mut nmea = Nmea::create_for_navigation(&[SentenceType::GGA]).unwrap();

    loop {
        let len = uart.read_until_idle(&mut buf).await.unwrap();
        let message = core::str::from_utf8(&buf[..len]).unwrap();

        match nmea.parse(message) {
            Ok(_) => defmt::info!("GPS: {:?}", nmea),
            Err(e) => defmt::error!("{:?}", Debug2Format(&e)),
        };

        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn debug_uart(mut debug_uart: DebugUartPort) {
    loop {
        debug_uart.write("hello world!".as_bytes()).await.unwrap();
        Timer::after_millis(2000).await;
    }
}
