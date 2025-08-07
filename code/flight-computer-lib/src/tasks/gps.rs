use defmt_or_log::{error, info, Debug2Format};
use embassy_sync::blocking_mutex::raw::RawMutex;
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{GpsMessage, GpsTopic};

use crate::device::gps::GpsDevice;

#[inline]
pub async fn gps_task<
    U, M, Tx,
    const DEPTH: usize,
> (
    uart: U,
    sd_card_sender: embassy_sync::channel::Sender<'static, M, GpsMessage, DEPTH>,
    postcard_sender: PostcardSender<Tx>,
) -> !
where
    U: embedded_io_async::Read,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut device = GpsDevice::init(uart).unwrap();
    let mut seq = 0_u32;

    loop {
        match device.parse_new_message().await {
            Ok(msg) => {
                // info!("GPS Message: {:?}", Debug2Format(&msg));

                if postcard_sender.publish::<GpsTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    // error!("Failed to publish GPS message to postcard client");
                }

                if sd_card_sender.try_send(msg).is_err() {
                    // error!("Failed to send GPS message to SD card task");
                }
            }, 
            // Err(e) => error!("Failed to read GPS: {:?}", Debug2Format(&e)),
            Err(_) => ()
        }
    }
}
