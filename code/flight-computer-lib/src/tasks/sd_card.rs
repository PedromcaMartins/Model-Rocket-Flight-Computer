use core::fmt::Debug;

use defmt_or_log::info;
use embassy_futures::select::{select4, Either4};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use embedded_sdmmc::{SdCard, VolumeIdx, VolumeManager};
use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};

use crate::model::system_status::SdCardSystemStatus;

const ID_OFFSET: u32 = 0x1000; // Offset for the ID partition
const MAX_DIRS: usize = 1;
const MAX_FILES: usize = 3;
const MAX_VOLUMES: usize = 1;

pub struct DummyTimeSource;
impl embedded_sdmmc::TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp::from_calendar(2025, 3, 7, 13, 23, 0).unwrap()
    }
}

#[inline]
pub async fn sd_card_task<
    S, D, W, O, M, 
    const DEPTH_ALTIMETER: usize,
    const DEPTH_GPS: usize,
    const DEPTH_IMU: usize,
> (
    sd_card: SdCard<S, D>,
    _sd_card_detect: W,
    _sd_card_status_led: O,
    status_signal: &'static Signal<M, SdCardSystemStatus>,
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
    let mut status = SdCardSystemStatus::default();

    let mut altimeter_msg_buf = [0u8; 1 << 7];
    let mut gps_msg_buf = [0u8; 1 << 7];
    let mut imu_msg_buf = [0u8; 1 << 10];

    while sd_card.get_card_type().is_none() {
        sd_card.mark_card_uninit();
        status.sd_card_not_recognized += 1;
        status_signal.signal(status.clone());
        Timer::after_millis(1_000).await;
    }

    info!("Sd Card type: {:?}", sd_card.get_card_type());
    if let Ok(bytes) = sd_card.num_bytes() {
        info!("Card size is {} GB", bytes >> 30);
    }

    // if sd_card_detect.wait_active().await.is_err() { 
    //     status.failed_to_sd_card_switch = true;
    // }

    let volume_mgr: VolumeManager<SdCard<S, D>, _, MAX_DIRS, MAX_FILES, MAX_VOLUMES> = VolumeManager::new_with_limits(sd_card, DummyTimeSource, ID_OFFSET);

    'setup: loop {
        status_signal.signal(status.clone());
        Timer::after_millis(1_000).await;

        let Ok(volume) = volume_mgr.open_volume(VolumeIdx(0)) else {
            status.failed_to_open_volume += 1;
            continue 'setup;
        };
        let Ok(root_dir) = volume.open_root_dir() else {
            status.failed_to_open_root_dir += 1;
            continue 'setup;
        };

        'open_file: loop {
            status_signal.signal(status.clone());
            Timer::after_millis(1_000).await;

            let Ok(altimeter_file) = root_dir.open_file_in_dir("alt.txt", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate) else {
                status.failed_to_open_altimeter_file += 1;
                continue 'open_file;
            };
            let Ok(gps_file) = root_dir.open_file_in_dir("gps.txt", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate) else {
                status.failed_to_open_gps_file += 1;
                continue 'open_file;
            };
            let Ok(imu_file) = root_dir.open_file_in_dir("imu.txt", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate) else {
                status.failed_to_open_imu_file += 1;
                continue 'open_file;
            };

            let flush_files_timeout = Instant::now() + Duration::from_millis(500);

            'run: loop {
                match select4 (
                    altimeter_receiver.receive(), 
                    gps_receiver.receive(), 
                    imu_receiver.receive(), 
                    Timer::at(flush_files_timeout),
                ).await {
                    Either4::First(altimeter_message) => {
                        // if sd_card_status_led.on().is_err() { status.failed_to_switch_led += 1 }
                        let Ok(len) = serde_json_core::to_slice(&altimeter_message, &mut altimeter_msg_buf) else { status.failed_to_serialize_altimeter_msg += 1; continue 'run; };
                        if altimeter_file.write(&altimeter_msg_buf[..len]).is_err() { status.failed_to_write_altimeter_msg += 1 }
                        // if sd_card_status_led.off().is_err() { status.failed_to_switch_led += 1 }
                        status.altimeter_message_written += 1;
                    },
                    Either4::Second(gps_message) => {
                        // if sd_card_status_led.on().is_err() { status.failed_to_switch_led += 1 }
                        let Ok(len) = serde_json_core::to_slice(&gps_message, &mut gps_msg_buf) else { status.failed_to_serialize_gps_msg += 1; continue 'run; };
                        if gps_file.write(&gps_msg_buf[..len]).is_err() { status.failed_to_write_gps_msg += 1 }
                        // if sd_card_status_led.off().is_err() { status.failed_to_switch_led += 1 }
                        status.gps_message_written += 1;
                    },
                    Either4::Third(imu_message) => {
                        // if sd_card_status_led.on().is_err() { status.failed_to_switch_led += 1 }
                        let Ok(len) = serde_json_core::to_slice(&imu_message, &mut imu_msg_buf) else { status.failed_to_serialize_imu_msg += 1; continue 'run; };
                        if imu_file.write(&imu_msg_buf[..len]).is_err() { status.failed_to_write_imu_msg += 1 }
                        // if sd_card_status_led.off().is_err() { status.failed_to_switch_led += 1 }
                        status.imu_message_written += 1;
                    },
                    Either4::Fourth(()) => {
                        if altimeter_file.flush().is_err() { status.failed_to_flush_altimeter_file += 1 }
                        if gps_file.flush().is_err() { status.failed_to_flush_gps_file += 1 }
                        if imu_file.flush().is_err() { status.failed_to_flush_imu_file += 1 }

                        // flush_files_timeout = Instant::now() + Duration::from_millis(500);
                        status.files_flushed += 1;

                        status_signal.signal(status.clone());
                    },
                }
            }
        }
    }
}
