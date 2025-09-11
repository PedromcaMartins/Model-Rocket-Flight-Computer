use embassy_futures::select::{select4, Either4};
use embassy_sync::channel::Receiver;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::{Duration, Ticker};
use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};
use defmt_or_log::info;

use crate::model::filesystem::log_filesystem::LogFileSystem;
use crate::model::filesystem::FileSystem;

#[inline]
pub async fn sd_card_task<
    FS, M, 
    const DEPTH_ALTIMETER_DATA: usize,
    const DEPTH_GPS_DATA: usize,
    const DEPTH_IMU_DATA: usize,
> (
    sd_card: FS,
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

    let mut flush_files_ticker = Ticker::every(Duration::from_millis(500));

    loop {
        let result = select4 (
            altimeter_receiver.receive(), 
            gps_receiver.receive(), 
            imu_receiver.receive(), 
            flush_files_ticker.next(),
        ).await;

        match result {
            Either4::First(altimeter_message) => {
                let res = log_filesystem.append_message(&altimeter_message);
                info!("SD Card: Logged altimeter message: {:?}", res);
            },
            Either4::Second(gps_message) => {
                let res = log_filesystem.append_message(&gps_message);
                info!("SD Card: Logged GPS message: {:?}", res);
            },
            Either4::Third(imu_message) => {
                let res = log_filesystem.append_message(&imu_message);
                info!("SD Card: Logged IMU message: {:?}", res);
            },
            Either4::Fourth(()) => {
                let res = log_filesystem.flush_all();
                info!("SD Card: Flushed all files: {:?}", res);

                // TODO: improve error FileSystemEvent to include Success + Failure states... 
                // if res.is_err() {
                //     error!("SD Card: Failed to flush files");
                // } else {
                //     info!("SD Card: Flushed all files");
                // }
            },
        }
    }
}
