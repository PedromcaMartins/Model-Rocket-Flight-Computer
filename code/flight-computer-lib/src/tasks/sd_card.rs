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

#[inline]
pub async fn sd_card_task<
    S, D, W, O, M, 
    const DEPTH_ALTIMETER: usize,
    const DEPTH_GPS: usize,
    const DEPTH_IMU: usize,
> (
    sd_card: SdCard<S, D>,
    mut sd_card_detect: W,
    mut sd_card_status_led: O,
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
    sd_card_detect.wait_active().await.unwrap();

    info!("Sd Card type: {:?}", sd_card.get_card_type());
    info!("Card size is {} GB", sd_card.num_bytes().unwrap() >> 30);

    let volume_mgr: VolumeManager<SdCard<S, D>, _, MAX_DIRS, MAX_FILES, MAX_VOLUMES> = VolumeManager::new_with_limits(sd_card, DummyTimeSource, ID_OFFSET);
    let volume = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let root_dir = volume.open_root_dir().unwrap();

    let altitude_file = root_dir.open_file_in_dir("Altitude.txt", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate).unwrap();
    let gps_file = root_dir.open_file_in_dir("Gps.txt", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate).unwrap();
    let imu_file = root_dir.open_file_in_dir("imu.txt", embedded_sdmmc::Mode::ReadWriteCreateOrTruncate).unwrap();
    let mut close_files_expired_time = Instant::now() + Duration::from_millis(500);

    loop {
        match select4 (
            altimeter_receiver.receive(), 
            gps_receiver.receive(), 
            imu_receiver.receive(), 
            Timer::at(close_files_expired_time),
        ).await {
            Either4::First(_altimeter_message) => {
                sd_card_status_led.on().unwrap();
                altitude_file.write(b"Altimeter Message!\r\n").unwrap();
                sd_card_status_led.off().unwrap();
            },
            Either4::Second(_gps_message) => {
                sd_card_status_led.on().unwrap();
                gps_file.write(b"GPS Message!\r\n").unwrap();
                sd_card_status_led.off().unwrap();
            },
            Either4::Third(_imu_message) => {
                sd_card_status_led.on().unwrap();
                imu_file.write(b"IMU Message!\r\n").unwrap();
                sd_card_status_led.off().unwrap();
            },
            Either4::Fourth(()) => {
                info!("Files flushed after timeout");

                altitude_file.flush().unwrap();
                gps_file.flush().unwrap();
                imu_file.flush().unwrap();

                close_files_expired_time = Instant::now() + Duration::from_millis(500);
            },
        }
    }
}
