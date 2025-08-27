use core::num::Wrapping;

use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{GpsMessage, GpsTopic};

use crate::{device::sensor::SensorDevice, model::system_status::GpsSystemStatus};

#[inline]
pub async fn gps_task<
    S, M, Tx,
    const DEPTH: usize,
> (
    mut gps: S,
    status_signal: &'static Signal<M, GpsSystemStatus>,
    sd_card_sender: embassy_sync::channel::Sender<'static, M, GpsMessage, DEPTH>,
    postcard_sender: PostcardSender<Tx>,
) -> !
where
    S: SensorDevice<DataMessage = GpsMessage>,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut status = GpsSystemStatus::default();

    let mut seq: Wrapping<u32> = Wrapping::default();

    let mut sensor_timeout = Instant::now();
    let mut status_timeout = Instant::now();

    loop {
        match select (
            Timer::at(sensor_timeout),
            Timer::at(status_timeout),
        ).await {
            Either::First(()) => {
                match gps.parse_new_message().await {
                    Err(_) => status.failed_to_parse_message += 1,
                    Ok(msg) => {
                        status.message_parsed += 1;

                        if postcard_sender.publish::<GpsTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                            seq += 1;
                        } else {
                            status.failed_to_publish_to_postcard += 1;
                        }

                        if sd_card_sender.try_send(msg).is_err() {
                            status.failed_to_publish_to_sd_card += 1;
                        }
                    }, 
                }

                sensor_timeout = Instant::now() + Duration::from_millis(50);
            },
            Either::Second(()) => {
                status_timeout = Instant::now() + Duration::from_millis(1_000);
                status_signal.signal(status.clone());
            },
        }
    }
}
