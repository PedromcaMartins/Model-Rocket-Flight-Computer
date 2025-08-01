#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

mod io_mapping;
use bmp280_ehal::BMP280;
use bno055::Bno055;
use io_mapping::IOMapping;

mod postcard_server;

use crate::{io_mapping::{Bmp280Port, Bno055Port, UbloxNeo7mPort}, postcard_server::spawn_postcard_server};

use {esp_backtrace as _, esp_println as _};

use embassy_executor::Spawner;

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


    spawner.must_spawn(bno055_task(bno055));
    spawner.must_spawn(bmp280_task(bmp280));
    spawner.must_spawn(gps_task(ublox_neo_7m));

    spawn_postcard_server(spawner, postcard_server_usb_driver).await;
}

#[embassy_executor::task]
async fn bno055_task(bno055: Bno055Port) {
    let bno055 = Bno055::new(bno055);

    flight_computer::tasks::bno055_task(bno055).await
}

#[embassy_executor::task]
async fn bmp280_task(bmp280: Bmp280Port) {
    let bmp280 = BMP280::new(bmp280).unwrap();

    flight_computer::tasks::bmp280_task(bmp280).await
}

#[embassy_executor::task]
async fn gps_task(gps: UbloxNeo7mPort) {
    flight_computer::tasks::gps_task(gps).await
}
