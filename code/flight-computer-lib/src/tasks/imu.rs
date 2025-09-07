use core::num::{Saturating, Wrapping};

use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender};
use embassy_time::{Duration, Instant, Timer};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{ImuMessage, ImuTopic};

use crate::{model::sensor_device::SensorDevice, error_sending_to_system_status, send_to_system_status};
use crate::model::system_status::ImuSystemStatus;

#[inline]
pub async fn imu_task<
    S, M, Tx,
    const DEPTH_STATUS: usize,
    const DEPTH_DATA: usize,
> (
    mut imu: S,
    status_sender: Sender<'static, M, Result<ImuSystemStatus, usize>, DEPTH_STATUS>,
    sd_card_sender: Sender<'static, M, ImuMessage, DEPTH_DATA>,
    postcard_sender: PostcardSender<Tx>,
) -> !
where
    S: SensorDevice<DataMessage = ImuMessage>,
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
                match imu.parse_new_message().await {
                    Ok(msg) => {
                        send_to_system_status!(status_sender, error_sending_status, ImuSystemStatus::MessageParsed);

                        if postcard_sender.publish::<ImuTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                            seq += 1;
                        } else { 
                            send_to_system_status!(status_sender, error_sending_status, ImuSystemStatus::FailedToPublishToPostcard);
                        }

                        if sd_card_sender.try_send(msg).is_err() {
                            send_to_system_status!(status_sender, error_sending_status, ImuSystemStatus::FailedToPublishToSdCard);
                        }
                    },
                    Err(_) => send_to_system_status!(status_sender, error_sending_status, ImuSystemStatus::FailedToParseMessage),
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
