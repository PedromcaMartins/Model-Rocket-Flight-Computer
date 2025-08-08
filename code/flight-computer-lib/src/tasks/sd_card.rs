use core::fmt::Debug;

use defmt_or_log::info;
use embassy_futures::select::{select4, Either4};
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::{Duration, Instant, Timer};
use embedded_sdmmc::{SdCard, VolumeIdx, VolumeManager};
use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};

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

#[derive(Debug, Clone, Default)]
pub struct SdCardSystemStatus {
    pub altimeter_message_written: u64,
    pub gps_message_written: u64,
    pub imu_message_written: u64,
    pub files_flushed: u64,

    pub failed_to_open_volume: u64,
    pub failed_to_open_root_dir: u64,
    pub failed_to_open_altitude_file: u64,
    pub failed_to_open_gps_file: u64,
    pub failed_to_open_imu_file: u64,
    pub failed_to_flush_altitude_file: u64,
    pub failed_to_flush_gps_file: u64,
    pub failed_to_flush_imu_file: u64,
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

    info!("Sd Card type: {:?}", sd_card.get_card_type());
    if let Ok(bytes) = sd_card.num_bytes() {
        info!("Card size is {} GB", bytes >> 30);
    }

    // if sd_card_detect.wait_active().await.is_err() { 
    //     status.failed_to_sd_card_switch = true;
    // }

    let volume_mgr: VolumeManager<SdCard<S, D>, _, MAX_DIRS, MAX_FILES, MAX_VOLUMES> = VolumeManager::new_with_limits(sd_card, DummyTimeSource, ID_OFFSET);

    'setup: loop {
        Timer::after_millis(1_000).await;

        let Ok(volume) = volume_mgr.open_volume(VolumeIdx(0)) else {
            status.failed_to_open_volume += 1;
            continue 'setup;
        };
        let Ok(root_dir) = volume.open_root_dir() else {
            status.failed_to_open_root_dir += 1;
            continue 'setup;
        };

        let Ok(altitude_file) = root_dir.open_file_in_dir("Altitude.txt", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate) else {
            status.failed_to_open_altitude_file += 1;
            continue 'setup;
        };
        let Ok(gps_file) = root_dir.open_file_in_dir("Gps.txt", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate) else {
            status.failed_to_open_gps_file += 1;
            continue 'setup;
        };
        let Ok(imu_file) = root_dir.open_file_in_dir("imu.txt", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate) else {
            status.failed_to_open_imu_file += 1;
            continue 'setup;
        };
        let mut close_files_expired_time = Instant::now() + Duration::from_millis(500);

        '_run: loop {
            match select4 (
                altimeter_receiver.receive(), 
                gps_receiver.receive(), 
                imu_receiver.receive(), 
                Timer::at(close_files_expired_time),
            ).await {
                Either4::First(_altimeter_message) => {
                    // if sd_card_status_led.on().is_err() { status.failed_to_switch_led += 1 }
                    if altitude_file.write(b"Altimeter Message!\r\n").is_err() { status.failed_to_open_altitude_file += 1 }
                    // if sd_card_status_led.off().is_err() { status.failed_to_switch_led += 1 }
                    status.altimeter_message_written += 1;
                },
                Either4::Second(_gps_message) => {
                    // if sd_card_status_led.on().is_err() { status.failed_to_switch_led += 1 }
                    if gps_file.write(b"GPS Message!\r\n").is_err() { status.failed_to_open_gps_file += 1 }
                    // if sd_card_status_led.off().is_err() { status.failed_to_switch_led += 1 }
                    status.gps_message_written += 1;
                },
                Either4::Third(_imu_message) => {
                    // if sd_card_status_led.on().is_err() { status.failed_to_switch_led += 1 }
                    if imu_file.write(b"IMU Message!\r\n").is_err() { status.failed_to_open_imu_file += 1 }
                    // if sd_card_status_led.off().is_err() { status.failed_to_switch_led += 1 }
                    status.imu_message_written += 1;
                },
                Either4::Fourth(()) => {
                    if altitude_file.flush().is_err() { status.failed_to_flush_altitude_file += 1 }
                    if gps_file.flush().is_err() { status.failed_to_flush_gps_file += 1 }
                    if imu_file.flush().is_err() { status.failed_to_flush_imu_file += 1 }

                    close_files_expired_time = Instant::now() + Duration::from_millis(500);
                    status.files_flushed += 1;
                },
            }
        }
    }
}
