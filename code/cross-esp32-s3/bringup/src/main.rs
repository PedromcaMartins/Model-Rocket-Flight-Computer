#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

mod io_mapping;
use embedded_sdmmc::{VolumeIdx, VolumeManager};
use io_mapping::IOMapping;
use postcard_rpc::header::VarHeader;

mod postcard_server;

use crate::{io_mapping::{ArmButtonPort, Bmp280Port, Bno055Port, DebugPort, ErrorLedPort, InitArmLedPort, RGBLedPort, RecoveryActivatedLedPort, SdCardDetectPort, SdCardInsertedLedPort, SdCardPort, UbloxNeo7mPort, WarningLedPort}, postcard_server::{spawn_postcard_server, Context}};

use {esp_backtrace as _, esp_println as _};

use embassy_executor::Spawner;

use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use bno055::{BNO055OperationMode, BNO055PowerMode, Bno055};

use defmt::{Debug2Format, info, error};
use embassy_time::{Delay, Instant, Timer};
use nmea::{Nmea, SentenceType};
use smart_leds::SmartLedsWriteAsync;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
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
        rgb_led,
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
    //     rgb_led,
    // ));

    // spawn_postcard_server(spawner, postcard_server_usb_driver).await;
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
        let pressure = bmp280.pressure().unwrap();
        #[allow(clippy::cast_possible_truncation)]
        let temperature = bmp280.temp().unwrap() as f32;

        info!("Pressure: {:?} Pa, Temperature: {:?} °C", pressure, temperature);

        Timer::after_millis(100).await;
    }
}

struct FakeTimeSource;
impl embedded_sdmmc::TimeSource for FakeTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp::from_calendar(2025, 3, 7, 13, 23, 0).unwrap()
    }
}

#[embassy_executor::task]
async fn sd_card_task(mut sd_card: SdCardPort, sd_card_detect: SdCardDetectPort, mut sd_card_status_led: SdCardInsertedLedPort) {
    for _ in 1..=4 {
        sd_card_status_led.toggle();
        Timer::after_secs(1).await;
    }

    info!("Sd Card Detect State: {:#?}", sd_card_detect.level());

    info!("Sd Card type {}", sd_card.get_card_type());

    // info!("Card size is {} bytes", sd_card.num_bytes().unwrap());
    let mut volume_mgr = VolumeManager::new(sd_card, FakeTimeSource);
    let volume = match volume_mgr.open_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to open volume: {:?}", e);
            return;
        }
    };
    info!("Volume: {:?}", volume);

    let root_dir = volume.open_root_dir().unwrap();
    let mut my_file = root_dir.open_file_in_dir("MY_FILE.TXT", embedded_sdmmc::Mode::ReadOnly).unwrap();
    while !my_file.is_eof() {
        let mut buffer = [0u8; 32];
        let num_read = my_file.read(&mut buffer).unwrap();
        for b in &buffer[0..num_read] {
            info!("{}", *b as char);
        }
    }
}

#[embassy_executor::task]
async fn gps_task(mut uart: UbloxNeo7mPort) {
    let mut buf = [0; nmea::SENTENCE_MAX_LEN];
    let mut nmea = Nmea::create_for_navigation(&[SentenceType::GGA]).unwrap();

    loop {
        if let Ok(len) = uart.read_async(&mut buf).await {
            if let Ok(message) = core::str::from_utf8(&buf[..len]) {
                match nmea.parse(message) {
                    Ok(_) => info!("GPS: {:?}", Debug2Format(&nmea)),
                    Err(e) => error!("ErroDebug2Format(&r): {:?}, Message: {}", Debug2Format(&e), message),
                }
            }
        }

        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn debug_uart_task(mut debug_port: DebugPort) {
    loop {
        debug_port.write_async(b"hello world!\r\n").await.unwrap();
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
    mut rgb_led: RGBLedPort,
) {
    // for _ in 1..4 {
    //     init_arm_led.toggle();
    //     recovery_activated_led.toggle();
    //     warning_led.toggle();
    //     error_led.toggle();
    //     Timer::after_secs(1).await;
    // }
    {
        use smart_leds::colors::*;

        for color in [BLUE, BLUE_VIOLET, SKY_BLUE, DARK_BLUE, ALICE_BLUE, CADET_BLUE, LIGHT_BLUE, ROYAL_BLUE, SLATE_BLUE, STEEL_BLUE, DODGER_BLUE, MEDIUM_BLUE, POWDER_BLUE, DEEP_SKY_BLUE, MIDNIGHT_BLUE, LIGHT_SKY_BLUE, CORNFLOWER_BLUE, DARK_SLATE_BLUE, LIGHT_STEEL_BLUE, MEDIUM_SLATE_BLUE] {
            rgb_led.write([color; 1]).await.unwrap();
            Timer::after_secs(1).await;
        }
    }
    for _ in 1..4 {
        arm_button.wait_for_rising_edge().await;
        Timer::after_secs(1).await;
    }
}

fn ping_handler(_context: &mut Context, _header: VarHeader, rqst: u32) -> u32 {
    info!("ping");
    rqst
}
