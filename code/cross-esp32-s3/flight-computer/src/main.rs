#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(unused_must_use)]

mod io_mapping;
use io_mapping::IOMapping;

mod postcard_server;

use crate::{io_mapping::{Bmp280Port, Bno055Port, UbloxNeo7mPort}, postcard_server::{init_postcard_server, server_task, AppTx}};

use bmp280_ehal::BMP280;
use bno055::Bno055;
use postcard_rpc::server::Sender;

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

    let server = init_postcard_server(spawner, postcard_server_usb_driver).await;

    // let arm_button_signal = Signal::new();
    // let altitude_signal = Signal::new();

    // spawner.must_spawn(bno055_task(bno055, server.sender()));
    // spawner.must_spawn(bmp280_task(bmp280, server.sender()));
    // spawner.must_spawn(gps_task(ublox_neo_7m, server.sender()));
    // spawner.must_spawn(finite_state_machine_task(

    // ));

    spawner.must_spawn(server_task(server));
}

#[embassy_executor::task]
    async fn bno055_task(bno055: Bno055Port, sender: Sender<AppTx>) {
    let bno055 = Bno055::new(bno055);

    flight_computer_lib::tasks::bno055_task(bno055, sender).await
}

#[embassy_executor::task]
async fn bmp280_task(bmp280: Bmp280Port, sender: Sender<AppTx>) {
    let bmp280 = BMP280::new(bmp280).unwrap();

    flight_computer_lib::tasks::bmp280_task(bmp280, sender).await
}

#[embassy_executor::task]
async fn gps_task(gps: UbloxNeo7mPort, sender: Sender<AppTx>) {
    flight_computer_lib::tasks::gps_task(gps, sender).await
}
