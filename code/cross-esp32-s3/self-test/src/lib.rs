#![no_std]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_sdmmc::{VolumeIdx, VolumeManager};

use board::{ArmButtonPort, Bmp280Port, Bno055Port, DebugPort, ErrorLedPort, InitArmLedPort, RGBLedPort, RecoveryActivatedLedPort, SdCardDetectPort, SdCardInsertedLedPort, SdCardPort, UbloxNeo7mPort, WarningLedPort};

use bmp280_ehal::{Config, Control, Filter, Oversampling, PowerMode, Standby, BMP280};
use bno055::{BNO055OperationMode, BNO055PowerMode, Bno055};

use defmt::{Debug2Format, info, error};
use embassy_time::{Delay, Instant, Timer};
use nmea::{Nmea, SentenceType};
use smart_leds::SmartLedsWriteAsync;

#[embassy_executor::task]
pub async fn bno055_task(bno055: Bno055Port) {
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
pub async fn bmp280_task(bmp280: Bmp280Port) {
    let mut bmp280 = BMP280::new(bmp280).unwrap();

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
pub async fn sd_card_task(sd_card: SdCardPort, sd_card_detect: SdCardDetectPort, mut sd_card_status_led: SdCardInsertedLedPort) {
    for _ in 1..=4 {
        sd_card_status_led.toggle();
        Timer::after_secs(1).await;
    }

    info!("Sd Card Detect State: {:#?}", sd_card_detect.level());

    info!("Sd Card type {}", sd_card.get_card_type());

    // info!("Card size is {} bytes", sd_card.num_bytes().unwrap());
    let volume_mgr = VolumeManager::new(sd_card, FakeTimeSource);
    let volume = match volume_mgr.open_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to open volume: {:?}", e);
            return;
        }
    };
    info!("Volume: {:?}", volume);

    let root_dir = volume.open_root_dir().unwrap();
    let my_file = root_dir.open_file_in_dir("MY_FILE.TXT", embedded_sdmmc::Mode::ReadOnly).unwrap();
    while !my_file.is_eof() {
        let mut buffer = [0u8; 32];
        let num_read = my_file.read(&mut buffer).unwrap();
        for b in &buffer[0..num_read] {
            info!("{}", *b as char);
        }
    }
}

#[embassy_executor::task]
pub async fn gps_task(mut uart: UbloxNeo7mPort) {
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
pub async fn debug_uart_task(mut debug_port: DebugPort) {
    loop {
        debug_port.write_async(b"hello world!\r\n").await.unwrap();
        Timer::after_millis(2000).await;
    }
}

#[embassy_executor::task]
pub async fn leds_buttons_task(
    mut _init_arm_led: InitArmLedPort,
    mut _recovery_activated_led: RecoveryActivatedLedPort,
    mut _warning_led: WarningLedPort,
    mut _error_led: ErrorLedPort,
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
