#![no_std]
#![no_main]
#![deny(unsafe_code)]

mod io_mapping;
use io_mapping::IOMapping;
use postcard_rpc::header::VarHeader;

mod postcard_server;

use crate::{io_mapping::{ArmButtonPort, Bmp280Port, Bno055Port, DebugPort, ErrorLedPort, InitArmLedPort, RecoveryActivatedLedPort, SdCardDetectPort, SdCardInsertedLedPort, SdCardPort, UbloxNeo7mPort, WarningLedPort}, postcard_server::{spawn_postcard_server, Context}};

use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;

use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use bno055::{BNO055OperationMode, BNO055PowerMode, Bno055};

use defmt::Debug2Format;
use embassy_stm32::time::Hertz;
use embassy_time::{Delay, Instant, Timer};
use nmea::{Nmea, SentenceType};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let IOMapping {
        bno055,
        bmp280,
        sd_card,
        sd_card_detect,
        sd_card_status_led,
        debug_port,
        ublox_neo_7m,
        postcard_server_usb_driver,
        init_arm_led,
        recovery_activated_led,
        warning_led,
        error_led,
        arm_button,
    } = IOMapping::init();

    // spawner.must_spawn(bno055_task(bno055));
    // spawner.must_spawn(bmp280_task(bmp280));
    // spawner.must_spawn(sd_card_task(sd_card, sd_card_detect, sd_card_status_led));
    // spawner.must_spawn(gps_task(ublox_neo_7m));
    // spawner.must_spawn(debug_uart_task(debug_port));
    // spawner.must_spawn(leds_buttons_task(
    //     init_arm_led,
    //     recovery_activated_led,
    //     warning_led,
    //     error_led,
    //     arm_button,
    // ));

    spawn_postcard_server(spawner, postcard_server_usb_driver).await;
}

#[embassy_executor::task]
async fn bno055_task(bno055: Bno055Port) {
    // The sensor has an initial startup time of 400ms - 650ms during which interaction with it will fail
    Timer::at(Instant::from_millis(650)).await;

    let mut delay = Delay;
    let mut bno055 = Bno055::new(bno055);

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
async fn bmp280_task(bmp280: Bmp280Port) {
    let mut bmp280 = BMP280::new(bmp280).unwrap();

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

        defmt::info!("Pressure: {:?} Pa, Temperature: {:?} °C", pressure, temperature);

        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn sd_card_task(mut sd_card: SdCardPort, sd_card_detect: SdCardDetectPort, mut sd_card_status_led: SdCardInsertedLedPort) {
    // Should print 400kHz for initialization
    defmt::info!("Configured clock: {}", sd_card.clock().0);

    let mut err = None;
    loop {
        match sd_card.init_card(Hertz::mhz(24)).await {
            Ok(()) => break,
            Err(e) => {
                if err != Some(e) {
                    defmt::error!("waiting for card error, retrying: {:?}", e);
                    err = Some(e);
                }
            }
        }
    }

    let sd_card_inner = defmt::unwrap!(sd_card.card());

    defmt::info!("Card: {:#?}", Debug2Format(sd_card_inner));
    defmt::info!("Clock: {}", sd_card.clock());
    defmt::info!("Sd Card Detect State: {:#?}", sd_card_detect.get_level());

    for _ in 1..=4 {
        sd_card_status_led.toggle();
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn gps_task(mut uart: UbloxNeo7mPort) {
    let mut buf = [0; nmea::SENTENCE_MAX_LEN];
    let mut nmea = Nmea::create_for_navigation(&[SentenceType::GGA]).unwrap();

    loop {
        if let Ok(len) = uart.read_until_idle(&mut buf).await {
            let message = core::str::from_utf8(&buf[..len]).unwrap();

            match nmea.parse(message) {
                Ok(_) => defmt::info!("GPS: {:?}", nmea),
                Err(e) => defmt::error!("Error: {:?}, Message: {}", Debug2Format(&e), message),
            }
        }

        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn debug_uart_task(mut debug_port: DebugPort) {
    loop {
        debug_port.write("hello world!\r\n".as_bytes()).await.unwrap();
        Timer::after_millis(2000).await;
    }
}

#[embassy_executor::task]
async fn leds_buttons_task(
    mut init_arm_led: InitArmLedPort,
    mut recovery_activated_led: RecoveryActivatedLedPort,
    mut warning_led: WarningLedPort,
    mut error_led: ErrorLedPort,
    mut arm_button: ArmButtonPort,
) {
    for _ in 1..4 {
        init_arm_led.toggle();
        recovery_activated_led.toggle();
        warning_led.toggle();
        error_led.toggle();
        Timer::after_secs(1).await;
    }
    for _ in 1..4 {
        arm_button.wait_for_rising_edge().await;
        Timer::after_secs(1).await;
    }
}

fn ping_handler(_context: &mut Context, _header: VarHeader, rqst: u32) -> u32 {
    defmt::info!("ping");
    rqst
}
