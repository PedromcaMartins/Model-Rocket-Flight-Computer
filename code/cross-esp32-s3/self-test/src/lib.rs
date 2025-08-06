#![no_std]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_sdmmc::{VolumeIdx, VolumeManager};

use board::{ArmButtonPort, Bmp280Port, Bno055Port, DebugPort, RGBLedPort, SdCardDetectPort, SdCardInsertedLedPort, SdCardPort, UbloxNeo7mPort};

use bmp280_ehal::BMP280;
use bno055::Bno055;

use defmt::{Debug2Format, info, error};
use embassy_time::Timer;
use flight_computer_lib::device::{bmp280::Bmp280Device, bno055::Bno055Device, gps::GpsDevice};
use smart_leds::SmartLedsWriteAsync;
use switch_hal::WaitSwitch;

pub async fn bno055_task(bno055: Bno055Port) {
    let bno055 = Bno055::new(bno055);
    let mut device = Bno055Device::init(bno055).await.unwrap();

    for _ in 1..4 {
        let msg = device.parse_new_message().unwrap();
        info!("Bno055 Message: {:?}", Debug2Format(&msg));
    }
}

pub async fn bmp280_task(bmp280: Bmp280Port) {
    let bmp280 = BMP280::new(bmp280).unwrap();
    let mut device = Bmp280Device::init(bmp280).unwrap();

    for _ in 1..4 {
        let msg = device.parse_new_message().unwrap();
        info!("Bmp280 Message: {:?}", Debug2Format(&msg));
    }
}

struct FakeTimeSource;
impl embedded_sdmmc::TimeSource for FakeTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp::from_calendar(2025, 3, 7, 13, 23, 0).unwrap()
    }
}

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

pub async fn gps_task(uart: UbloxNeo7mPort) {
    let mut device = GpsDevice::init(uart).unwrap();

    for _ in 1..4 {
        let msg = device.parse_new_message().await.unwrap();
        info!("Gps Message: {:?}", Debug2Format(&msg));
    }
}

pub async fn debug_uart_task(mut debug_port: DebugPort) {
    loop {
        debug_port.write_async(b"hello world!\r\n").await.unwrap();
        Timer::after_millis(2000).await;
    }
}

pub async fn leds_buttons_task(
    mut arm_button: ArmButtonPort,
    mut rgb_led: RGBLedPort,
) {
    {
        use smart_leds::colors::*;

        for color in [BLUE, BLUE_VIOLET, SKY_BLUE, DARK_BLUE, ALICE_BLUE, CADET_BLUE, LIGHT_BLUE, ROYAL_BLUE, SLATE_BLUE, STEEL_BLUE, DODGER_BLUE, MEDIUM_BLUE, POWDER_BLUE, DEEP_SKY_BLUE, MIDNIGHT_BLUE, LIGHT_SKY_BLUE, CORNFLOWER_BLUE, DARK_SLATE_BLUE, LIGHT_STEEL_BLUE, MEDIUM_SLATE_BLUE] {
            rgb_led.write([color; 1]).await.unwrap();
            Timer::after_secs(1).await;
        }
    }

    // Wait for arm button presses todo!
    for _ in 1..4 {
        arm_button.wait_active().await.unwrap();
        Timer::after_secs(1).await;
    }
}
