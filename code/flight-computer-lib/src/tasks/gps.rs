use defmt_or_log::{error, info, Debug2Format};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::GpsTopic;

use crate::device::gps::GpsDevice;

#[inline]
pub async fn gps_task<U, Tx>(
    uart: U,
    sender: PostcardSender<Tx>,
) -> !
where
    U: embedded_io_async::Read,
    Tx: WireTx,
{
    let mut device = GpsDevice::init(uart).unwrap();
    let mut seq = 0_u32;

    loop {
        match device.parse_new_message().await {
            Ok(msg) => {
                info!("GPS Message: {:?}", Debug2Format(&msg));

                if sender.publish::<GpsTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    error!("Failed to publish GPS message");
                }
            }, 
            Err(e) => error!("Failed to read GPS: {:?}", Debug2Format(&e)),
        }
    }
}
