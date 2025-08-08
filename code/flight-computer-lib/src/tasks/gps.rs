use core::convert::Infallible;

use embassy_sync::blocking_mutex::raw::RawMutex;
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{GpsMessage, GpsTopic};

use crate::device::gps::GpsDevice;

#[derive(Debug, Clone, Default)]
pub struct GpsSystemStatus {
    pub message_parsed: u64,
    pub failed_to_initialize_device: u64,
    pub failed_to_parse_message: u64,
    pub failed_to_publish_to_postcard: u64,
    pub failed_to_publish_to_sd_card: u64,
}

#[inline]
pub async fn gps_task<
    U, M, Tx,
    const DEPTH: usize,
> (
    uart: U,
    sd_card_sender: embassy_sync::channel::Sender<'static, M, GpsMessage, DEPTH>,
    postcard_sender: PostcardSender<Tx>,
) -> Result<Infallible, ()>
where
    U: embedded_io_async::Read,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut status = GpsSystemStatus::default();

    let Ok(mut device) = GpsDevice::init(uart) else {
        status.failed_to_initialize_device += 1;
        return Err(());
    };
    let mut seq = 0_u32;

    loop {
        match device.parse_new_message().await {
            Err(_) => status.failed_to_parse_message += 1,
            Ok(msg) => {
                status.message_parsed += 1;

                if postcard_sender.publish::<GpsTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    status.failed_to_publish_to_postcard += 1;
                }

                if sd_card_sender.try_send(msg).is_err() {
                    status.failed_to_publish_to_sd_card += 1;
                }
            }, 
        }
    }
}
