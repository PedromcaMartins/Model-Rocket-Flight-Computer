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
use board::{ArmButtonPeripheral, Bmp280Peripheral, Bno055Peripheral, Board, DeploymentPeripheral, SdCardDetectPeripheral, SdCardInsertedLedPeripheral, SdCardPeripheral, UbloxNeo7mPeripheral};

use bmp280_ehal::BMP280;
use bno055::Bno055;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::{self, Channel}, signal::Signal, watch::{self, Watch}};
use flight_computer_lib::{config::{ApogeeDetectorConfig, TouchdownDetectorConfig}, impls::{deployment_switch::DeploymentSwitch, sd_card::SdCardFatFS, bmp280::Bmp280Device, bno055::Bno055Device, gps::GpsDevice}};
use postcard_rpc::server::Sender as PostcardSender;
use static_cell::ConstStaticCell;
use telemetry_messages::{AltimeterMessage, Altitude, FlightState, GpsMessage, ImuMessage};

use {esp_backtrace as _, esp_println as _};

use embassy_executor::Spawner;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const ID_OFFSET: u32 = 0x1000; // Offset for the ID partition

type EmbassySyncRawMutex = CriticalSectionRawMutex;

static LATEST_ALTITUDE_SIGNAL: ConstStaticCell<Signal<EmbassySyncRawMutex, Altitude>> = ConstStaticCell::new(Signal::new());

const FLIGHT_STATE_WATCH_CONSUMERS: usize = 2;
static FLIGHT_STATE_WATCH: ConstStaticCell<Watch<EmbassySyncRawMutex, FlightState, FLIGHT_STATE_WATCH_CONSUMERS>> = ConstStaticCell::new(Watch::new());

const ALTIMETER_SD_CARD_CHANNEL_DEPTH: usize = 10;
const GPS_SD_CARD_CHANNEL_DEPTH: usize = 10;
const IMU_SD_CARD_CHANNEL_DEPTH: usize = 10;

static ALTIMETER_SD_CARD_CHANNEL: ConstStaticCell<Channel<EmbassySyncRawMutex, AltimeterMessage, ALTIMETER_SD_CARD_CHANNEL_DEPTH>> = ConstStaticCell::new(Channel::new());
static GPS_SD_CARD_CHANNEL:       ConstStaticCell<Channel<EmbassySyncRawMutex, GpsMessage, GPS_SD_CARD_CHANNEL_DEPTH>> = ConstStaticCell::new(Channel::new());
static IMU_SD_CARD_CHANNEL:       ConstStaticCell<Channel<EmbassySyncRawMutex, ImuMessage, IMU_SD_CARD_CHANNEL_DEPTH>> = ConstStaticCell::new(Channel::new());

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let Board { 
        bno055, 
        bmp280, 
        sd_card, 
        sd_card_detect, 
        sd_card_status_led, 
        debug_peripheral: _, 
        ublox_neo_7m, 
        postcard_server_usb_driver, 
        arm_button,
        rgb_led: _,
        deployment,
    } = Board::init();

    let server = init_postcard_server(spawner, postcard_server_usb_driver).await;

    let latest_altitude_signal = LATEST_ALTITUDE_SIGNAL.take();

    let flight_state_watch = FLIGHT_STATE_WATCH.take();

    let altimeter_sd_card_channel = ALTIMETER_SD_CARD_CHANNEL.take();
    let gps_sd_card_channel = GPS_SD_CARD_CHANNEL.take();
    let imu_sd_card_channel = IMU_SD_CARD_CHANNEL.take();

    spawner.must_spawn(imu_task(bno055, imu_sd_card_channel.sender(), server.sender()));
    spawner.must_spawn(altimeter_task(bmp280, latest_altitude_signal, altimeter_sd_card_channel.sender(), server.sender()));
    spawner.must_spawn(gps_task(ublox_neo_7m, gps_sd_card_channel.sender(), server.sender()));
    spawner.must_spawn(sd_card_task(
        sd_card, 
        sd_card_detect, 
        sd_card_status_led, 
        altimeter_sd_card_channel.receiver(), 
        gps_sd_card_channel.receiver(), 
        imu_sd_card_channel.receiver()
    ));
    spawner.must_spawn(finite_state_machine_task(arm_button, deployment, latest_altitude_signal, flight_state_watch.sender()));

    spawner.must_spawn(server_task(server));
}

#[embassy_executor::task]
async fn imu_task(
    bno055: Bno055Peripheral, 
    sd_card_sender: channel::Sender<'static, EmbassySyncRawMutex, ImuMessage, IMU_SD_CARD_CHANNEL_DEPTH>,
    postcard_sender: PostcardSender<AppTx>,
) {
    let bno055 = Bno055::new(bno055);
    let bno055 = Bno055Device::init(bno055).await.unwrap();

    flight_computer_lib::tasks::imu_task(bno055, sd_card_sender, postcard_sender).await
}

#[embassy_executor::task]
async fn altimeter_task(
    bmp280: Bmp280Peripheral, 
    latest_altitude_signal: &'static Signal<EmbassySyncRawMutex, Altitude>,
    sd_card_sender: channel::Sender<'static, EmbassySyncRawMutex, AltimeterMessage, ALTIMETER_SD_CARD_CHANNEL_DEPTH>,
    postcard_sender: PostcardSender<AppTx>,
) {
    let bmp280 = BMP280::new(bmp280).unwrap();
    let bmp280 = Bmp280Device::init(bmp280).unwrap();

    flight_computer_lib::tasks::altimeter_task(bmp280, latest_altitude_signal, sd_card_sender, postcard_sender).await
}

#[embassy_executor::task]
async fn gps_task(
    gps: UbloxNeo7mPeripheral, 
    sd_card_sender: channel::Sender<'static, EmbassySyncRawMutex, GpsMessage, GPS_SD_CARD_CHANNEL_DEPTH>,
    postcard_sender: PostcardSender<AppTx>,
) {
    let gps = GpsDevice::init(gps).unwrap();

    flight_computer_lib::tasks::gps_task(gps, sd_card_sender, postcard_sender).await
}

#[embassy_executor::task]
async fn finite_state_machine_task(
    arm_button: ArmButtonPeripheral,
    deployment: DeploymentPeripheral,
    latest_altitude_signal: &'static Signal<EmbassySyncRawMutex, Altitude>,
    flight_state_sender: watch::Sender<'static, EmbassySyncRawMutex, FlightState, FLIGHT_STATE_WATCH_CONSUMERS>,
) {
    let deployment = DeploymentSwitch::new(deployment);

    flight_computer_lib::tasks::finite_state_machine_task(
        arm_button, 
        deployment, 
        latest_altitude_signal, 
        ApogeeDetectorConfig::default(), 
        TouchdownDetectorConfig::default(),
        flight_state_sender, 
    ).await
}

#[embassy_executor::task]
async fn sd_card_task(
    sd_card: SdCardPeripheral,
    sd_card_detect: SdCardDetectPeripheral,
    sd_card_status_led: SdCardInsertedLedPeripheral,
    altimeter_receiver: channel::Receiver<'static, EmbassySyncRawMutex, AltimeterMessage, ALTIMETER_SD_CARD_CHANNEL_DEPTH>,
    gps_receiver: channel::Receiver<'static, EmbassySyncRawMutex, GpsMessage, GPS_SD_CARD_CHANNEL_DEPTH>,
    imu_receiver: channel::Receiver<'static, EmbassySyncRawMutex, ImuMessage, IMU_SD_CARD_CHANNEL_DEPTH>,
) -> ! {
    let sd_card = SdCardFatFS::init::<ID_OFFSET>(sd_card).unwrap();

    flight_computer_lib::tasks::sd_card_task(
        sd_card, 
        sd_card_detect,
        sd_card_status_led,
        altimeter_receiver, 
        gps_receiver, 
        imu_receiver
    ).await
}
