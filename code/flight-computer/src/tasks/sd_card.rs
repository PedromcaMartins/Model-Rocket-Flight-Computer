use embassy_futures::select::{select4, Either4};
use embassy_time::Ticker;
use switch_hal::OutputSwitch;
use defmt_or_log::{debug, error, info};

use crate::{config::LogFileSystemConfig, core::{storage::log_filesystem::LogFileSystem, trace::{TraceAsync, TraceSync}}, interfaces::FileSystem, sync::{ALTIMETER_SD_CARD_CHANNEL, GPS_SD_CARD_CHANNEL, IMU_SD_CARD_CHANNEL}};

#[inline]
pub async fn sd_card_task<
    FS, O,
> (
    sd_card: FS,
    mut sd_card_led: O,
) -> !
where
    FS: FileSystem,
    O: OutputSwitch,
{
    let altimeter_receiver = ALTIMETER_SD_CARD_CHANNEL.receiver();
    let gps_receiver = GPS_SD_CARD_CHANNEL.receiver();
    let imu_receiver = IMU_SD_CARD_CHANNEL.receiver();

    let trace = TraceSync::start("sd_card_task_init");

    let mut log_filesystem = LogFileSystem::new(sd_card);
    let res = log_filesystem.create_unique_files();
    info!("SD Card: Created unique log files: {:?}", res);

    let mut flush_files_ticker = Ticker::every(LogFileSystemConfig::FLUSH_FILES_TICKER_PERIOD);

    drop(trace);
    loop {
        let mut trace = TraceAsync::start("sd_card_task_loop");

        trace.before_await();
        let result = select4 (
            altimeter_receiver.receive(), 
            gps_receiver.receive(), 
            imu_receiver.receive(), 
            flush_files_ticker.next(),
        ).await;
        trace.after_await();

        if sd_card_led.on().is_err() { error!("SD Card: Status Led error") }

        match result {
            Either4::First(altimeter_message) => {
                let res = log_filesystem.append_message(&altimeter_message);
                debug!("SD Card: Logged altimeter message: {:?}", res);
            },
            Either4::Second(gps_message) => {
                let res = log_filesystem.append_message(&gps_message);
                debug!("SD Card: Logged GPS message: {:?}", res);
            },
            Either4::Third(imu_message) => {
                let res = log_filesystem.append_message(&imu_message);
                debug!("SD Card: Logged IMU message: {:?}", res);
            },
            Either4::Fourth(()) => {
                let res = log_filesystem.flush_all();
                debug!("SD Card: Flushed all files: {:?}", res);

                // TODO: improve error FileSystemEvent to include Success + Failure states... 
                // if res.is_err() {
                //     error!("SD Card: Failed to flush files");
                // } else {
                //     debug!("SD Card: Flushed all files");
                // }
            },
        }

        if sd_card_led.off().is_err() { error!("SD Card: Status Led error") }
    }
}
