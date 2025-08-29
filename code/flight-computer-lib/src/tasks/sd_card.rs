use core::num::Saturating;

use embassy_futures::select::{select, select4, Either, Either4};
use embassy_sync::channel::{Receiver, Sender};
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::{Duration, Instant, Timer};
use enum_map::Enum;
use static_cell::ConstStaticCell;
use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};

use crate::model::filesystem::{FileSystem, LogDataType};
use crate::model::system_status::SdCardSystemStatus;
use crate::{error_sending_to_system_status, send_to_system_status};

const ALTIMETER_TMP_BUF_SIZE: usize = 1 << 7;
const GPS_TMP_BUF_SIZE: usize = 1 << 7;
const IMU_TMP_BUF_SIZE: usize = 1 << 10;

static ALTIMETER_TMP_BUF: ConstStaticCell<[u8; ALTIMETER_TMP_BUF_SIZE]> = ConstStaticCell::new([0u8; ALTIMETER_TMP_BUF_SIZE]);
static GPS_TMP_BUF: ConstStaticCell<[u8; GPS_TMP_BUF_SIZE]> = ConstStaticCell::new([0u8; GPS_TMP_BUF_SIZE]);
static IMU_TMP_BUF: ConstStaticCell<[u8; IMU_TMP_BUF_SIZE]> = ConstStaticCell::new([0u8; IMU_TMP_BUF_SIZE]);

#[inline]
pub async fn sd_card_task<
    FS, M, 
    const DEPTH_STATUS: usize,
    const DEPTH_ALTIMETER_DATA: usize,
    const DEPTH_GPS_DATA: usize,
    const DEPTH_IMU_DATA: usize,
> (
    mut sd_card: FS,
    status_sender: Sender<'static, M, Result<SdCardSystemStatus, usize>, DEPTH_STATUS>,
    altimeter_receiver: Receiver<'static, M, AltimeterMessage, DEPTH_ALTIMETER_DATA>,
    gps_receiver: Receiver<'static, M, GpsMessage, DEPTH_GPS_DATA>,
    imu_receiver: Receiver<'static, M, ImuMessage, DEPTH_IMU_DATA>,
) -> !
where
    FS: FileSystem,
    M: RawMutex + 'static,
{
    let altimeter_tmp_buf = ALTIMETER_TMP_BUF.take();
    let gps_tmp_buf = GPS_TMP_BUF.take();
    let imu_tmp_buf = IMU_TMP_BUF.take();

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
                let res = sd_card.append_message(LogDataType::Altimeter, &altimeter_message, altimeter_tmp_buf);
                send_to_system_status!(status_sender, error_sending_status, SdCardSystemStatus::FileSystemEvent(LogDataType::Altimeter, res));
            },
            Either::First(Either4::Second(gps_message)) => {
                let res = sd_card.append_message(LogDataType::Gps, &gps_message, gps_tmp_buf);
                send_to_system_status!(status_sender, error_sending_status, SdCardSystemStatus::FileSystemEvent(LogDataType::Gps, res));
            },
            Either::First(Either4::Third(imu_message)) => {
                let res = sd_card.append_message(LogDataType::Imu, &imu_message, imu_tmp_buf);
                send_to_system_status!(status_sender, error_sending_status, SdCardSystemStatus::FileSystemEvent(LogDataType::Imu, res));
            },
            Either::First(Either4::Fourth(())) => {
                const LOG_DATA_TYPES: [LogDataType; LogDataType::LENGTH] = [LogDataType::Altimeter, LogDataType::Gps, LogDataType::Imu];

                for log_data_type in LOG_DATA_TYPES {
                    let res = sd_card.reopen_file(log_data_type);
                    send_to_system_status!(status_sender, error_sending_status, SdCardSystemStatus::FileSystemEvent(log_data_type, res));
                }

                flush_files_timeout = Instant::now() + Duration::from_millis(500);
            },
            Either::Second(()) => {
                error_sending_to_system_status!(status_sender, error_sending_status);
                status_timeout = Instant::now() + Duration::from_secs(1);
            },
        }
    }
}
