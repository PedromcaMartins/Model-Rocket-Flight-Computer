#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(unused_must_use)]

mod postcard_server;

use crate::{postcard_server::{init_postcard_server, server_task, AppTx}};
use board::{ArmButtonPeripheral, Bmp280Peripheral, Bno055Peripheral, Board, UbloxNeo7mPeripheral};

use bmp280_ehal::BMP280;
use bno055::Bno055;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use postcard_rpc::server::Sender;
use static_cell::ConstStaticCell;
use uom::si::f64::Length;

use {esp_backtrace as _, esp_println as _};

use embassy_executor::Spawner;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

static ARM_BUTTON_SIGNAL: ConstStaticCell<Signal<CriticalSectionRawMutex, ()>>   = ConstStaticCell::new(Signal::new());
static ALTITUDE_SIGNAL: ConstStaticCell<Signal<CriticalSectionRawMutex, Length>> = ConstStaticCell::new(Signal::new());

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let Board { 
        bno055, 
        bmp280, 
        sd_card: _, 
        sd_card_detect: _, 
        sd_card_status_led: _, 
        debug_port: _, 
        ublox_neo_7m, 
        postcard_server_usb_driver, 
        arm_button,
        rgb_led: _,
    } = Board::init();

    let server = init_postcard_server(spawner, postcard_server_usb_driver).await;

    let arm_button_signal = ARM_BUTTON_SIGNAL.take();
    let altitude_signal = ALTITUDE_SIGNAL.take();

    spawner.must_spawn(bno055_task(bno055, server.sender()));
    spawner.must_spawn(bmp280_task(bmp280, altitude_signal, server.sender()));
    spawner.must_spawn(gps_task(ublox_neo_7m, server.sender()));
    spawner.must_spawn(arm_button_task(arm_button, arm_button_signal));
    spawner.must_spawn(finite_state_machine_task(arm_button_signal, altitude_signal));

    spawner.must_spawn(server_task(server));
}

#[embassy_executor::task]
    async fn bno055_task(bno055: Bno055Peripheral, sender: Sender<AppTx>) -> ! {
    let bno055 = Bno055::new(bno055);

    flight_computer_lib::tasks::bno055_task(bno055, sender).await
}

#[embassy_executor::task]
async fn bmp280_task(
    bmp280: Bmp280Peripheral, 
    altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    sender: Sender<AppTx>,
) -> ! {
    let bmp280 = BMP280::new(bmp280).unwrap();

    flight_computer_lib::tasks::bmp280_task(bmp280, altitude_signal, sender).await
}

#[embassy_executor::task]
async fn gps_task(gps: UbloxNeo7mPeripheral, sender: Sender<AppTx>) -> ! {
    flight_computer_lib::tasks::gps_task(gps, sender).await
}

#[embassy_executor::task]
async fn arm_button_task(
    arm_button: ArmButtonPeripheral,
    arm_button_signal: &'static Signal<CriticalSectionRawMutex, ()>,
) -> ! {
    flight_computer_lib::tasks::arm_button_task(arm_button, arm_button_signal).await
}

#[embassy_executor::task]
async fn finite_state_machine_task(
    arm_button_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
) {
    flight_computer_lib::tasks::finite_state_machine_task(arm_button_signal, altitude_signal).await
}
