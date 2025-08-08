use core::{convert::Infallible, fmt::Debug};

use bno055::Bno055;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::Timer;
use embedded_hal::i2c::{I2c, SevenBitAddress};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{ImuMessage, ImuTopic};

use crate::device::bno055::Bno055Device;

#[derive(Debug, Clone, Default)]
pub struct ImuSystemStatus {
    pub message_parsed: u64,
    pub failed_to_initialize_device: u64,
    pub failed_to_parse_message: u64,
    pub failed_to_publish_to_postcard: u64,
    pub failed_to_publish_to_sd_card: u64,
}

#[inline]
pub async fn bno055_task<
    I, E, M, Tx,
    const DEPTH: usize,
> (
    bno055: Bno055<I>,
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
    let mut seq = 0_u32;

    loop {
        match device.parse_new_message() {
            Err(_) => status.failed_to_parse_message += 1,
            Ok(msg) => {
                status.message_parsed += 1;

                if postcard_sender.publish::<ImuTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    status.failed_to_publish_to_postcard += 1;
                }

                if sd_card_sender.try_send(msg).is_err() {
                    status.failed_to_publish_to_sd_card += 1;
                }
            },
        }

        Timer::after_millis(50).await;
    }
}
