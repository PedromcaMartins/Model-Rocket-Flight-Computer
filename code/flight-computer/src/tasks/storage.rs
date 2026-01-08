use embassy_futures::select::{select, Either};
use embassy_time::Ticker;
use defmt_or_log::{debug, error, info};

use crate::{config::StorageConfig, core::{storage::Storage, trace::{TraceAsync, TraceSync}}, interfaces::{FileSystem, Led}, sync::RECORD_TO_STORAGE_CHANNEL};

#[inline]
pub async fn storage_task<FS, LED>(filesystem: FS, mut filesystem_led: LED) -> !
where
    FS: FileSystem,
    LED: Led,
{
    let trace = TraceSync::start("storage_task_init");
    let receiver = RECORD_TO_STORAGE_CHANNEL.receiver();

    let mut storage = Storage::new(filesystem)
        .await.expect("Storage: Initialization failed");
    info!("Storage: Created unique log files");

    let mut flush_files_ticker = Ticker::every(StorageConfig::FLUSH_FILES_TICKER_PERIOD);

    drop(trace);
    loop {
        let mut trace = TraceAsync::start("storage_task_loop");

        trace.before_await();
        let result = select (
            receiver.receive(), 
            flush_files_ticker.next(),
        ).await;
        trace.after_await();

        if filesystem_led.on().await.is_err() { error!("Storage: Status Led error") }

        match result {
            Either::First(record) => {
                let res = storage.append_record(&record).await;
                debug!("Storage: Logged record: {:?}", res);
            },
            Either::Second(()) => {
                let res = storage.flush().await;
                debug!("Storage: Flushed file: {:?}", res);
            },
        }

        if filesystem_led.off().await.is_err() { error!("Storage: Status Led error") }
    }
}
