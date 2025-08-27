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
use board::{ArmButtonPeripheral, Bmp280Peripheral, Bno055Peripheral, Board, SdCardDetectPeripheral, SdCardInsertedLedPeripheral, SdCardPeripheral, UbloxNeo7mPeripheral};

use bmp280_ehal::BMP280;
use bno055::Bno055;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::{Channel, Receiver, Sender}, signal::Signal, watch::{self, Watch}};
use flight_computer_lib::{device::{sd_card::SdCardDevice, sensor::{bmp280::Bmp280Device, bno055::Bno055Device, gps::GpsDevice}}, model::system_status::{AltimeterSystemStatus, ArmButtonSystemStatus, FiniteStateMachineStatus, GpsSystemStatus, ImuSystemStatus, SdCardSystemStatus}};
use postcard_rpc::server::Sender as PostcardSender;
use static_cell::ConstStaticCell;
use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};
use uom::si::f64::Length;

use {esp_backtrace as _, esp_println as _};

use embassy_executor::Spawner;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

type EmbassySyncRawMutex = CriticalSectionRawMutex;

static ARM_BUTTON_SIGNAL: ConstStaticCell<Signal<EmbassySyncRawMutex, ()>>   = ConstStaticCell::new(Signal::new());
static ALTITUDE_SIGNAL: ConstStaticCell<Signal<EmbassySyncRawMutex, Length>> = ConstStaticCell::new(Signal::new());

static ALTITUDE_STATUS_SIGNAL:   ConstStaticCell<Signal<EmbassySyncRawMutex, AltimeterSystemStatus>> =    ConstStaticCell::new(Signal::new());
static ARM_BUTTON_STATUS_SIGNAL: ConstStaticCell<Signal<EmbassySyncRawMutex, ArmButtonSystemStatus>> =    ConstStaticCell::new(Signal::new());
static IMU_STATUS_SIGNAL:        ConstStaticCell<Signal<EmbassySyncRawMutex, ImuSystemStatus>> =          ConstStaticCell::new(Signal::new());
static GPS_STATUS_SIGNAL:        ConstStaticCell<Signal<EmbassySyncRawMutex, GpsSystemStatus>> =          ConstStaticCell::new(Signal::new());
static SD_CARD_STATUS_SIGNAL:    ConstStaticCell<Signal<EmbassySyncRawMutex, SdCardSystemStatus>> =       ConstStaticCell::new(Signal::new());

const FSM_WATCH_N_CONSUMERS: usize = 2;
static FSM_STATUS_WATCH: ConstStaticCell<Watch<EmbassySyncRawMutex, FiniteStateMachineStatus, FSM_WATCH_N_CONSUMERS>> = ConstStaticCell::new(Watch::new());

const ALTIMETER_CHANNEL_DEPTH: usize = 10;
const GPS_CHANNEL_DEPTH: usize = 10;
const IMU_CHANNEL_DEPTH: usize = 10;

static ALTIMETER_SD_CARD_CHANNEL: ConstStaticCell<Channel<EmbassySyncRawMutex, AltimeterMessage, ALTIMETER_CHANNEL_DEPTH>> = ConstStaticCell::new(Channel::new());
static GPS_SD_CARD_CHANNEL:       ConstStaticCell<Channel<EmbassySyncRawMutex, GpsMessage, GPS_CHANNEL_DEPTH>> = ConstStaticCell::new(Channel::new());
static IMU_SD_CARD_CHANNEL:       ConstStaticCell<Channel<EmbassySyncRawMutex, ImuMessage, IMU_CHANNEL_DEPTH>> = ConstStaticCell::new(Channel::new());

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
    } = Board::init();

    let server = init_postcard_server(spawner, postcard_server_usb_driver).await;

    let arm_button_signal = ARM_BUTTON_SIGNAL.take();
    let altitude_signal = ALTITUDE_SIGNAL.take();

    let altitude_status_signal = ALTITUDE_STATUS_SIGNAL.take();
    let arm_button_status_signal = ARM_BUTTON_STATUS_SIGNAL.take();
    let imu_status_signal = IMU_STATUS_SIGNAL.take();
    let gps_status_signal = GPS_STATUS_SIGNAL.take();
    let sd_card_status_signal = SD_CARD_STATUS_SIGNAL.take();

    let fsm_status_watch = FSM_STATUS_WATCH.take();

    let altimeter_sd_card_channel = ALTIMETER_SD_CARD_CHANNEL.take();
    let gps_sd_card_channel = GPS_SD_CARD_CHANNEL.take();
    let imu_sd_card_channel = IMU_SD_CARD_CHANNEL.take();

    spawner.must_spawn(bno055_task(bno055, imu_status_signal, imu_sd_card_channel.sender(), server.sender()));
    spawner.must_spawn(bmp280_task(bmp280, altitude_signal, altitude_status_signal, altimeter_sd_card_channel.sender(), server.sender()));
    spawner.must_spawn(gps_task(ublox_neo_7m, gps_status_signal, gps_sd_card_channel.sender(), server.sender()));
    spawner.must_spawn(sd_card_task(
        sd_card, 
        sd_card_detect, 
        sd_card_status_led, 
        sd_card_status_signal, 
        altimeter_sd_card_channel.receiver(), 
        gps_sd_card_channel.receiver(), 
        imu_sd_card_channel.receiver()
    ));
    spawner.must_spawn(arm_button_task(arm_button, arm_button_signal, arm_button_status_signal));
    spawner.must_spawn(finite_state_machine_task(arm_button_signal, altitude_signal, fsm_status_watch.sender()));
    spawner.must_spawn(system_status_task(
        altitude_status_signal, 
        arm_button_status_signal, 
        imu_status_signal, 
        gps_status_signal, 
        sd_card_status_signal, 
        fsm_status_watch.receiver().unwrap(),
    ));

    spawner.must_spawn(server_task(server));
}

#[embassy_executor::task]
async fn bno055_task(
    bno055: Bno055Peripheral, 
    imu_status_signal: &'static Signal<EmbassySyncRawMutex, ImuSystemStatus>,
    imu_sd_card_sender: Sender<'static, EmbassySyncRawMutex, ImuMessage, IMU_CHANNEL_DEPTH>,
    postcard_sender: PostcardSender<AppTx>,
) {
    let bno055 = Bno055::new(bno055);
    let bno055 = Bno055Device::init(bno055).await.unwrap();

    flight_computer_lib::tasks::bno055_task(bno055, imu_status_signal, imu_sd_card_sender, postcard_sender).await
}

#[embassy_executor::task]
async fn bmp280_task(
    bmp280: Bmp280Peripheral, 
    altitude_signal: &'static Signal<EmbassySyncRawMutex, Length>,
    altitude_status_signal: &'static Signal<EmbassySyncRawMutex, AltimeterSystemStatus>,
    altimeter_sd_card_sender: Sender<'static, EmbassySyncRawMutex, AltimeterMessage, ALTIMETER_CHANNEL_DEPTH>,
    postcard_sender: PostcardSender<AppTx>,
) {
    let bmp280 = BMP280::new(bmp280).unwrap();
    let bmp280 = Bmp280Device::init(bmp280).unwrap();

    flight_computer_lib::tasks::bmp280_task(bmp280, altitude_signal, altitude_status_signal, altimeter_sd_card_sender, postcard_sender).await
}

#[embassy_executor::task]
async fn gps_task(
    gps: UbloxNeo7mPeripheral, 
    gps_status_signal: &'static Signal<EmbassySyncRawMutex, GpsSystemStatus>,
    gps_sd_card_sender: Sender<'static, EmbassySyncRawMutex, GpsMessage, GPS_CHANNEL_DEPTH>,
    postcard_sender: PostcardSender<AppTx>,
) {
    let gps = GpsDevice::init(gps).unwrap();

    flight_computer_lib::tasks::gps_task(gps, gps_status_signal, gps_sd_card_sender, postcard_sender).await
}

#[embassy_executor::task]
async fn arm_button_task(
    arm_button: ArmButtonPeripheral,
    arm_button_signal: &'static Signal<EmbassySyncRawMutex, ()>,
    arm_button_status_signal: &'static Signal<EmbassySyncRawMutex, ArmButtonSystemStatus>,
) -> ! {
    flight_computer_lib::tasks::arm_button_task(arm_button, arm_button_signal, arm_button_status_signal).await
}

#[embassy_executor::task]
async fn finite_state_machine_task(
    arm_button_signal: &'static Signal<EmbassySyncRawMutex, ()>,
    altitude_signal: &'static Signal<EmbassySyncRawMutex, Length>,
    fsm_status_sender: watch::Sender<'static, EmbassySyncRawMutex, FiniteStateMachineStatus, FSM_WATCH_N_CONSUMERS>,
) {
    flight_computer_lib::tasks::finite_state_machine_task(arm_button_signal, altitude_signal, fsm_status_sender).await
}

#[embassy_executor::task]
async fn sd_card_task(
    sd_card: SdCardPeripheral,
    sd_card_detect: SdCardDetectPeripheral,
    sd_card_status_led: SdCardInsertedLedPeripheral,
    sd_card_status_signal: &'static Signal<EmbassySyncRawMutex, SdCardSystemStatus>,
    altimeter_receiver: Receiver<'static, EmbassySyncRawMutex, AltimeterMessage, ALTIMETER_CHANNEL_DEPTH>,
    gps_receiver: Receiver<'static, EmbassySyncRawMutex, GpsMessage, GPS_CHANNEL_DEPTH>,
    imu_receiver: Receiver<'static, EmbassySyncRawMutex, ImuMessage, IMU_CHANNEL_DEPTH>,
) -> ! {
    const ID_OFFSET: u32 = 0x1000; // Offset for the ID partition

    const MAX_DIRS: usize = 1;
    const MAX_FILES: usize = 3;
    const MAX_VOLUMES: usize = 1;

    static ALTIMETER_FILENAME: &str = "ALT.TXT";
    static GPS_FILENAME: &str = "GPS.TXT";
    static IMU_FILENAME: &str = "IMU.TXT";

    let sd_card: SdCardDevice<
        _, _, _, _, 
        MAX_DIRS, 
        MAX_FILES, 
        MAX_VOLUMES,
    > = SdCardDevice::init::<ID_OFFSET>(
        sd_card, 
        sd_card_detect, 
        sd_card_status_led, 
        &ALTIMETER_FILENAME, 
        &GPS_FILENAME, 
        &IMU_FILENAME, 
    ).unwrap();

    flight_computer_lib::tasks::sd_card_task(
        sd_card, 
        sd_card_status_signal,
        altimeter_receiver, 
        gps_receiver, 
        imu_receiver
    ).await
}

#[embassy_executor::task]
async fn system_status_task(
    altitude_status_signal: &'static Signal<EmbassySyncRawMutex, AltimeterSystemStatus>,
    arm_button_status_signal: &'static Signal<EmbassySyncRawMutex, ArmButtonSystemStatus>,
    imu_status_signal: &'static Signal<EmbassySyncRawMutex, ImuSystemStatus>,
    gps_status_signal: &'static Signal<EmbassySyncRawMutex, GpsSystemStatus>,
    sd_card_status_signal: &'static Signal<EmbassySyncRawMutex, SdCardSystemStatus>,
    fsm_status_watch_receiver: watch::Receiver<'static, EmbassySyncRawMutex, FiniteStateMachineStatus, FSM_WATCH_N_CONSUMERS>,
) -> ! {
    flight_computer_lib::tasks::system_status_task(
        altitude_status_signal,
        arm_button_status_signal,
        imu_status_signal,
        gps_status_signal,
        sd_card_status_signal,
        fsm_status_watch_receiver
    ).await
}
