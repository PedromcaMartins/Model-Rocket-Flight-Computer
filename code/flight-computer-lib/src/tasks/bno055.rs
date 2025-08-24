use core::{convert::Infallible, fmt::Debug, num::Wrapping};

use bno055::Bno055;
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::i2c::{I2c, SevenBitAddress};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{ImuMessage, ImuTopic};

use crate::{device::bno055::Bno055Device, model::system_status::ImuSystemStatus};

#[inline]
pub async fn bno055_task<
    I, E, M, Tx,
    const DEPTH: usize,
> (
    bno055: Bno055<I>,
    status_signal: &'static Signal<M, ImuSystemStatus>,
    sd_card_sender: embassy_sync::channel::Sender<'static, M, ImuMessage, DEPTH>,
    postcard_sender: PostcardSender<Tx>,
) -> Result<Infallible, ()>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut status = ImuSystemStatus::default();

    let Ok(mut device) = Bno055Device::init(bno055).await else {
        status.failed_to_initialize_device += 1;
        return Err(());
    };
    let mut seq: Wrapping<u32> = Wrapping::default();

    let mut sensor_timeout = Instant::now();
    let mut status_timeout = Instant::now();

    loop {
        match select (
            Timer::at(sensor_timeout),
            Timer::at(status_timeout),
        ).await {
            Either::First(()) => {
                match device.parse_new_message() {
                    Err(_) => status.failed_to_parse_message += 1,
                    Ok(msg) => {
                        status.message_parsed += 1;

                        if postcard_sender.publish::<ImuTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
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
