#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use board::Board;
use self_test::{bmp280_task, bno055_task, debug_uart_task, gps_task, leds_buttons_task, sd_card_task};

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
        debug_port, 
        ublox_neo_7m, 
        postcard_server_usb_driver: _, 
        init_arm_led: _, 
        recovery_activated_led: _, 
        warning_led: _, 
        error_led: _, 
        arm_button,
        rgb_led,
    } = Board::init();


    bno055_task(bno055).await;
    bmp280_task(bmp280).await;
    sd_card_task(sd_card, sd_card_detect, sd_card_status_led).await;
    gps_task(ublox_neo_7m).await;
    debug_uart_task(debug_port).await;
    leds_buttons_task(
        arm_button,
        rgb_led,
    ).await;
}
