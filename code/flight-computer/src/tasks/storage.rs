use embassy_futures::select::{Either4, select4};
use embassy_time::{Timer, Ticker, with_timeout};
use crate::log::{trace, error, info, warn};
use proto::{RecordData, flight_state::FlightState};
use core::{future::Future, pin::Pin, task::Poll};

use crate::{config::StorageConfig, core::storage::Storage, interfaces::{FileSystem, Led}, sync::{RECORD_TO_STORAGE_CHANNEL, FLIGHT_STATE_WATCH}};

#[inline]
pub async fn storage_task<FS, LED>(filesystem: FS, mut led: LED)
where
    FS: FileSystem,
    LED: Led,
{
    let receiver = RECORD_TO_STORAGE_CHANNEL.receiver();

    let mut storage = Storage::new(filesystem)
        .await.expect("Storage: Initialization failed");
    info!("Storage: Created unique log files");

    let mut flush_files_ticker = Ticker::every(StorageConfig::FLUSH_FILES_TICK_INTERVAL);
    let mut flight_state_receiver = FLIGHT_STATE_WATCH.receiver()
        .expect("Storage: Not enough flight state consumers");

    let mut hold_timer = HoldTimer::new();

    loop {
        let result = select4(
            receiver.receive(),
            flush_files_ticker.next(),
            flight_state_receiver.changed(),
            &mut hold_timer,
        ).await;

        led.on().await.unwrap_or_else(|e| error!("Storage: Status Led error: {:?}", e));

        match result {
            Either4::First(record) => {
                match with_timeout(StorageConfig::WRITE_TIMEOUT, storage.append_record(&record)).await {
                    Err(_) => error!("Storage: Timed out writing record"),
                    Ok(res) => trace!("Storage: Logged record: {:?}", res),
                }
            },
            Either4::Second(()) => {
                match with_timeout(StorageConfig::FLUSH_TIMEOUT, storage.flush()).await {
                    Err(_) => warn!("Storage: Timed out flushing file"),
                    Ok(res) => trace!("Storage: Flushed file: {:?}", res),
                }
            },
            Either4::Third(record) => {
                if hold_timer.is_running() {
                    warn!("Storage: Timer already running, on flight state change: {:?}", record.payload());
                }
                else if matches!(record.payload(), RecordData::FlightState(FlightState::Touchdown)) {
                    info!("Storage: Touchdown detected, starting {}-second hold timer", StorageConfig::TOUCHDOWN_HOLD_DURATION.as_secs());
                    hold_timer.start();
                }
            },
            Either4::Fourth(()) => {
                info!("Storage: Final flush");
                match with_timeout(StorageConfig::FLUSH_TIMEOUT, storage.flush()).await {
                    Err(_) => warn!("Storage: Final flush timed out"),
                    Ok(_) => (),
                }
                info!("Storage: Exiting");

                led.off().await.unwrap_or_else(|e| error!("Storage: Status Led error: {:?}", e));
                return;
            },
        }

        led.off().await.unwrap_or_else(|e| error!("Storage: Status Led error: {:?}", e));
    }
}

struct HoldTimer(Option<Timer>);

impl HoldTimer {
    const fn new() -> Self {
        Self(None)
    }

    fn start(&mut self) {
        self.0 = Some(Timer::after(StorageConfig::TOUCHDOWN_HOLD_DURATION));
    }

    const fn is_running(&self) -> bool {
        self.0.is_some()
    }
}

impl Future for HoldTimer {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        self.get_mut().0.as_mut().map_or(Poll::Pending, |timer| Pin::new(timer).poll(cx))
    }
}
