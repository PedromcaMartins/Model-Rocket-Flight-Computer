use core::num::{Saturating, Wrapping};

use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{AltimeterMessage, AltimeterTopic};
use uom::si::f32::Length;

use crate::{model::sensor_device::SensorDevice, error_sending_to_system_status, send_to_system_status};
use crate::model::system_status::AltimeterSystemStatus;

#[inline]
pub async fn altimeter_task<
    S, M, Tx, 
    const DEPTH_STATUS: usize,
    const DEPTH_DATA: usize,
> (
    mut altimeter: S,
    latest_altitude_signal: &'static Signal<M, Length>,
    status_sender: Sender<'static, M, Result<AltimeterSystemStatus, usize>, DEPTH_STATUS>,
    sd_card_sender: Sender<'static, M, AltimeterMessage, DEPTH_DATA>,
    postcard_sender: PostcardSender<Tx>,
) -> !
where
    S: SensorDevice<DataMessage = AltimeterMessage>,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut seq: Wrapping<u32> = Wrapping::default();
    let mut error_sending_status = Saturating::default();

    let mut sensor_timeout = Instant::now();
    let mut status_timeout = Instant::now();

    loop {
        match select (
            Timer::at(sensor_timeout),
            Timer::at(status_timeout),
        ).await {
            Either::First(()) => {
                match altimeter.parse_new_message().await {
                    Ok(msg) => {
                        send_to_system_status!(status_sender, error_sending_status, AltimeterSystemStatus::MessageParsed);

                        if postcard_sender.publish::<AltimeterTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                            seq += 1;
                        } else { 
                            send_to_system_status!(status_sender, error_sending_status, AltimeterSystemStatus::FailedToPublishToPostcard);
                        }

                        latest_altitude_signal.signal(msg.altitude);

                        if sd_card_sender.try_send(msg).is_err() {
                            send_to_system_status!(status_sender, error_sending_status, AltimeterSystemStatus::FailedToPublishToSdCard);
                        }
                    },
                    Err(_) => send_to_system_status!(status_sender, error_sending_status, AltimeterSystemStatus::FailedToParseMessage),
                }

                sensor_timeout = Instant::now() + Duration::from_millis(50);
            },
            Either::Second(()) => {
                error_sending_to_system_status!(status_sender, error_sending_status);
                status_timeout = Instant::now() + Duration::from_secs(1);
            },
        }
    }
}
