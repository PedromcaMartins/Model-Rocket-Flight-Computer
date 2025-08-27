use core::fmt::Debug;

use defmt_or_log::error;
use embassy_futures::select::{select4, Either4};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use embedded_sdmmc::SdCard;
use static_cell::ConstStaticCell;
use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};

use crate::model::system_status::SdCardSystemStatus;
use crate::device::sd_card::{LogFileData, SdCardDevice};

const ID_OFFSET: u32 = 0x1000; // Offset for the ID partition
const MAX_DIRS: usize = 1;
const MAX_FILES: usize = 3;
const MAX_VOLUMES: usize = 1;

static ALTIMETER_FILENAME: &str = "ALT.TXT";
static GPS_FILENAME: &str = "GPS.TXT";
static IMU_FILENAME: &str = "IMU.TXT";

const ALTIMETER_TMP_BUF_SIZE: usize = 1 << 7;
const GPS_TMP_BUF_SIZE: usize = 1 << 7;
const IMU_TMP_BUF_SIZE: usize = 1 << 10;

static ALTIMETER_TMP_BUF: ConstStaticCell<[u8; ALTIMETER_TMP_BUF_SIZE]> = ConstStaticCell::new([0u8; ALTIMETER_TMP_BUF_SIZE]);
static GPS_TMP_BUF: ConstStaticCell<[u8; GPS_TMP_BUF_SIZE]> = ConstStaticCell::new([0u8; GPS_TMP_BUF_SIZE]);
static IMU_TMP_BUF: ConstStaticCell<[u8; IMU_TMP_BUF_SIZE]> = ConstStaticCell::new([0u8; IMU_TMP_BUF_SIZE]);

#[inline]
pub async fn sd_card_task<
    S, D, W, O, M, 
    const DEPTH_ALTIMETER: usize,
    const DEPTH_GPS: usize,
    const DEPTH_IMU: usize,
> (
    sd_card: SdCard<S, D>,
    sd_card_detect: W,
    sd_card_status_led: O,
    _status_signal: &'static Signal<M, SdCardSystemStatus>,
    altimeter_receiver: embassy_sync::channel::Receiver<'static, M, AltimeterMessage, DEPTH_ALTIMETER>,
    gps_receiver: embassy_sync::channel::Receiver<'static, M, GpsMessage, DEPTH_GPS>,
    imu_receiver: embassy_sync::channel::Receiver<'static, M, ImuMessage, DEPTH_IMU>,
) -> !
where
    S: embedded_hal::spi::SpiDevice,
    D: embedded_hal::delay::DelayNs,
    W: switch_hal::WaitSwitch<Error: Debug>,
    O: switch_hal::OutputSwitch<Error: Debug>,
    M: RawMutex + 'static,
{
    let mut _status = SdCardSystemStatus::default();

    let mut device: SdCardDevice<W, O, SdCard<S, D>, &'static str, MAX_DIRS, MAX_FILES, MAX_VOLUMES> 
        = SdCardDevice::init::<ID_OFFSET>(
        sd_card, 
        sd_card_detect, 
        sd_card_status_led, 
        &ALTIMETER_FILENAME, 
        &GPS_FILENAME, 
        &IMU_FILENAME, 
    ).unwrap();
    // ) {
    //     Ok(device) => device,
    //     Err(_err) => {
    //         error!("Failed to initialize SD Card. Retrying in 1 second...");
    //         Timer::after(Duration::from_secs(1)).await;
    //     }
    // };

    let altimeter_tmp_buf = ALTIMETER_TMP_BUF.take();
    let gps_tmp_buf = GPS_TMP_BUF.take();
    let imu_tmp_buf = IMU_TMP_BUF.take();
    
    let mut flush_files_timeout = Instant::now() + Duration::from_millis(500);

    loop {
        let result = select4 (
            altimeter_receiver.receive(), 
            gps_receiver.receive(), 
            imu_receiver.receive(), 
            Timer::at(flush_files_timeout),
        ).await;

        // if sd_card_status_led.on().is_err() { status.failed_to_switch_led += 1 }

        match result {
            Either4::First(altimeter_message) => {
                if let Err(err) = device.append_message(LogFileData::Altimeter, &altimeter_message, altimeter_tmp_buf) {
                    error!("Failed to append altimeter message to SD Card: {:?}", err);
                }
            },
            Either4::Second(gps_message) => {
                if let Err(err) = device.append_message(LogFileData::Gps, &gps_message, gps_tmp_buf) {
                    error!("Failed to append GPS message to SD Card: {:?}", err);
                }
            },
            Either4::Third(imu_message) => {
                if let Err(err) = device.append_message(LogFileData::Imu, &imu_message, imu_tmp_buf) {
                    error!("Failed to append IMU message to SD Card: {:?}", err);
                }
            },
            Either4::Fourth(()) => {
                if let Err(err) = device.flush_all_files() {
                    error!("Failed to flush SD Card files: {:?}", err);
                }

                flush_files_timeout = Instant::now() + Duration::from_millis(500);
            },
        }

        // if sd_card_status_led.off().is_err() { status.failed_to_switch_led += 1 }
    }
}
