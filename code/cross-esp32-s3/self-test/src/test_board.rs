#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use board::Board;
use self_test::{bmp280_test, bno055_test, debug_uart_test, gps_test, leds_buttons_test, sd_card_test};

use {esp_backtrace as _, esp_println as _};

use embassy_executor::Spawner;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let Board { 
        bno055, 
        bmp280, 
        sd_card, 
        sd_card_detect, 
        sd_card_status_led, 
        debug_peripheral, 
        ublox_neo_7m, 
        postcard_server_usb_driver: _, 
        arm_button,
        rgb_led,
        deployment: _,
    } = Board::init();


    bno055_test(bno055).await;
    bmp280_test(bmp280).await;
    sd_card_test(sd_card, sd_card_detect, sd_card_status_led).await;
    gps_test(ublox_neo_7m).await;
    debug_uart_test(debug_peripheral).await;
    leds_buttons_test(
        arm_button,
        rgb_led,
    ).await;
}
