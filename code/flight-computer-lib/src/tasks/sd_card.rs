use defmt_or_log::{error, Debug2Format};
use embassy_futures::select::{select4, Either4};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use static_cell::ConstStaticCell;
use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};

use crate::model::filesystem::{FileSystem, LogDataType};
use crate::model::system_status::SdCardSystemStatus;

const ALTIMETER_TMP_BUF_SIZE: usize = 1 << 7;
const GPS_TMP_BUF_SIZE: usize = 1 << 7;
const IMU_TMP_BUF_SIZE: usize = 1 << 10;

static ALTIMETER_TMP_BUF: ConstStaticCell<[u8; ALTIMETER_TMP_BUF_SIZE]> = ConstStaticCell::new([0u8; ALTIMETER_TMP_BUF_SIZE]);
static GPS_TMP_BUF: ConstStaticCell<[u8; GPS_TMP_BUF_SIZE]> = ConstStaticCell::new([0u8; GPS_TMP_BUF_SIZE]);
static IMU_TMP_BUF: ConstStaticCell<[u8; IMU_TMP_BUF_SIZE]> = ConstStaticCell::new([0u8; IMU_TMP_BUF_SIZE]);

#[inline]
pub async fn sd_card_task<
    FS, M, 
    const DEPTH_ALTIMETER: usize,
    const DEPTH_GPS: usize,
    const DEPTH_IMU: usize,
> (
    mut sd_card: FS,
    _status_signal: &'static Signal<M, SdCardSystemStatus>,
    altimeter_receiver: embassy_sync::channel::Receiver<'static, M, AltimeterMessage, DEPTH_ALTIMETER>,
    gps_receiver: embassy_sync::channel::Receiver<'static, M, GpsMessage, DEPTH_GPS>,
    imu_receiver: embassy_sync::channel::Receiver<'static, M, ImuMessage, DEPTH_IMU>,
) -> !
where
    FS: FileSystem,
    M: RawMutex + 'static,
{
    let mut _status = SdCardSystemStatus::default();

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

        match result {
            Either4::First(altimeter_message) => {
                if let Err(err) = sd_card.append_message(LogDataType::Altimeter, &altimeter_message, altimeter_tmp_buf) {
                    error!("Failed to append altimeter message to SD Card: {:?}", Debug2Format(&err));
                }
            },
            Either4::Second(gps_message) => {
                if let Err(err) = sd_card.append_message(LogDataType::Gps, &gps_message, gps_tmp_buf) {
                    error!("Failed to append GPS message to SD Card: {:?}", Debug2Format(&err));
                }
            },
            Either4::Third(imu_message) => {
                if let Err(err) = sd_card.append_message(LogDataType::Imu, &imu_message, imu_tmp_buf) {
                    error!("Failed to append IMU message to SD Card: {:?}", Debug2Format(&err));
                }
            },
            Either4::Fourth(()) => {
                if let Err(err) = sd_card.flush_all_files() {
                    error!("Failed to flush SD Card files: {:?}", Debug2Format(&err));
                }

                flush_files_timeout = Instant::now() + Duration::from_millis(500);
            },
        }
    }
}
