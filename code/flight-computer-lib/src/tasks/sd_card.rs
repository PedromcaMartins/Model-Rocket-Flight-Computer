use core::num::Saturating;

use embassy_futures::select::{select, select4, Either, Either4};
use embassy_sync::channel::{Receiver, Sender};
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::{Duration, Instant, Timer};
use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};

use crate::model::filesystem::log_filesystem::LogFileSystem;
use crate::model::filesystem::{FileSystem, LogDataType};
use crate::model::system_status::SdCardSystemStatus;
use crate::{error_sending_to_system_status, send_to_system_status};

#[inline]
pub async fn sd_card_task<
    FS, M, 
    const DEPTH_STATUS: usize,
    const DEPTH_ALTIMETER_DATA: usize,
    const DEPTH_GPS_DATA: usize,
    const DEPTH_IMU_DATA: usize,
> (
    sd_card: FS,
    status_sender: Sender<'static, M, Result<SdCardSystemStatus, usize>, DEPTH_STATUS>,
    altimeter_receiver: Receiver<'static, M, AltimeterMessage, DEPTH_ALTIMETER_DATA>,
    gps_receiver: Receiver<'static, M, GpsMessage, DEPTH_GPS_DATA>,
    imu_receiver: Receiver<'static, M, ImuMessage, DEPTH_IMU_DATA>,
) -> !
where
    FS: FileSystem,
    M: RawMutex + 'static,
{
    let mut log_filesystem = LogFileSystem::new(sd_card);
    log_filesystem.create_unique_files();

    let mut error_sending_status = Saturating::default();

    let mut flush_files_timeout = Instant::now() + Duration::from_millis(500);
    let mut status_timeout = Instant::now();

    loop {
        let result = select (
            select4 (
                altimeter_receiver.receive(), 
                gps_receiver.receive(), 
                imu_receiver.receive(), 
                Timer::at(flush_files_timeout),
            ),
            Timer::at(status_timeout),
        ).await;

        match result {
            Either::First(Either4::First(altimeter_message)) => {
                let res = log_filesystem.append_message(&altimeter_message);
                send_to_system_status!(status_sender, error_sending_status, SdCardSystemStatus::FileSystemEvent(LogDataType::Altimeter, res));
            },
            Either::First(Either4::Second(gps_message)) => {
                let res = log_filesystem.append_message(&gps_message);
                send_to_system_status!(status_sender, error_sending_status, SdCardSystemStatus::FileSystemEvent(LogDataType::Gps, res));
            },
            Either::First(Either4::Third(imu_message)) => {
                let res = log_filesystem.append_message(&imu_message);
                send_to_system_status!(status_sender, error_sending_status, SdCardSystemStatus::FileSystemEvent(LogDataType::Imu, res));
            },
            Either::First(Either4::Fourth(())) => {
                let _res = log_filesystem.flush_all();
                send_to_system_status!(status_sender, error_sending_status, SdCardSystemStatus::Other);

                flush_files_timeout = Instant::now() + Duration::from_millis(500);
            },
            Either::Second(()) => {
                error_sending_to_system_status!(status_sender, error_sending_status);
                status_timeout = Instant::now() + Duration::from_secs(1);
            },
        }
    }
}
