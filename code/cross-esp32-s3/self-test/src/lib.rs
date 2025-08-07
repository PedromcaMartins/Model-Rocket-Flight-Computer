#![no_std]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_sdmmc::{VolumeIdx, VolumeManager};

use board::{ArmButtonPeripheral, Bmp280Peripheral, Bno055Peripheral, DebugPeripheral, RGBLedPeripheral, SdCardDetectPeripheral, SdCardInsertedLedPeripheral, SdCardPeripheral, UbloxNeo7mPeripheral};

use bmp280_ehal::BMP280;
use bno055::Bno055;

use defmt::{Debug2Format, info, error};
use embassy_time::Timer;
use flight_computer_lib::device::{bmp280::Bmp280Device, bno055::Bno055Device, gps::GpsDevice};
use smart_leds::SmartLedsWriteAsync;
use switch_hal::{InputSwitch, StatefulOutputSwitch, WaitSwitch};

pub async fn bno055_test(bno055: Bno055Peripheral) {
    info!("Bno055 Test Started");

    let bno055 = Bno055::new(bno055);
    let mut device = Bno055Device::init(bno055).await.unwrap();

    for _ in 1..4 {
        let msg = device.parse_new_message().unwrap();
        info!("Bno055 Message: {:?}", Debug2Format(&msg));
    }

    info!("Bno055 Test Completed");
}

pub async fn bmp280_test(bmp280: Bmp280Peripheral) {
    info!("Bmp280 Test Started");

    let bmp280 = BMP280::new(bmp280).unwrap();
    let mut device = Bmp280Device::init(bmp280).unwrap();

    for _ in 1..4 {
        let msg = device.parse_new_message().unwrap();
        info!("Bmp280 Message: {:?}", Debug2Format(&msg));
    }

    info!("Bmp280 Test Completed");
}

struct DummyTimeSource;
impl embedded_sdmmc::TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp::from_calendar(2025, 3, 7, 13, 23, 0).unwrap()
    }
}

pub async fn sd_card_test(sd_card: SdCardPeripheral, mut sd_card_detect: SdCardDetectPeripheral, mut sd_card_status_led: SdCardInsertedLedPeripheral) {
    info!("Sd Card Test Started");

    for _ in 1..4 {
        sd_card_status_led.toggle().unwrap();
        Timer::after_secs(1).await;
    }

    info!("Sd Card Detect State: {}", if sd_card_detect.is_active().unwrap() { "active" } else { "inactive" });

    info!("Sd Card type {}", sd_card.get_card_type());

    // info!("Card size is {} bytes", sd_card.num_bytes().unwrap());
    let volume_mgr = VolumeManager::new(sd_card, DummyTimeSource);
    let volume = match volume_mgr.open_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to open volume: {:?}", e);
            return;
        }
    };
    info!("Volume: {:?}", volume);

    let root_dir = volume.open_root_dir().unwrap();

    let my_file = root_dir.open_file_in_dir("MY_FILE.TXT", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate).unwrap();
    my_file.write(b"Hello World!\r\n").unwrap();
    my_file.close().unwrap();

    let my_file = root_dir.open_file_in_dir("MY_FILE.TXT", embedded_sdmmc::Mode::ReadOnly).unwrap();
    while !my_file.is_eof() {
        let mut buffer = [0u8; 32];
        let num_read = my_file.read(&mut buffer).unwrap();
        for b in &buffer[0..num_read] {
            info!("{}", *b as char);
        }
    }

    info!("Sd Card Test Completed");
}

pub async fn gps_test(uart: UbloxNeo7mPeripheral) {
    info!("GPS Test Started");

    let mut device = GpsDevice::init(uart).unwrap();

    for _ in 1..4 {
        loop {
            let msg = match device.parse_new_message().await {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to parse GPS message: {:?}", Debug2Format(&e));
                    continue;
                }
            };
            info!("Gps Message: {:?}", Debug2Format(&msg));
            break;
        }
    }

    info!("GPS Test Completed");
}

pub async fn debug_uart_test(mut debug_peripheral: DebugPeripheral) {
    info!("Debug UART Test Started");

    for _ in 1..4 {
        debug_peripheral.write_async(b"hello world!\r\n").await.unwrap();
    }

    info!("Debug UART Test Completed");
}

pub async fn leds_buttons_test(
    mut arm_button: ArmButtonPeripheral,
    mut rgb_led: RGBLedPeripheral,
) {
    info!("LEDs and Buttons Test Started");

    {
        use smart_leds::colors::*;

        for color in [BLUE, BLUE_VIOLET, SKY_BLUE, DARK_BLUE, ALICE_BLUE, CADET_BLUE, LIGHT_BLUE, ROYAL_BLUE, SLATE_BLUE, STEEL_BLUE, DODGER_BLUE, MEDIUM_BLUE, POWDER_BLUE, DEEP_SKY_BLUE, MIDNIGHT_BLUE, LIGHT_SKY_BLUE, CORNFLOWER_BLUE, DARK_SLATE_BLUE, LIGHT_STEEL_BLUE, MEDIUM_SLATE_BLUE, BLACK] {
            rgb_led.write([color; 1]).await.unwrap();
            Timer::after_secs(1).await;
        }
    }

    // Wait for arm button presses todo!
    for _ in 1..4 {
        arm_button.wait_active().await.unwrap();
        Timer::after_secs(1).await;
    }

    info!("LEDs and Buttons Test Completed");
}
