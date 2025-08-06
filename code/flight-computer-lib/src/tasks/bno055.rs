use core::fmt::Debug;

use defmt_or_log::{info, error};
use bno055::Bno055;
use defmt_or_log::Debug2Format;
use embedded_hal::i2c::{I2c, SevenBitAddress};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::ImuTopic;

use crate::device::bno055::Bno055Device;

#[inline]
pub async fn bno055_task<I, E, Tx>(
    bno055: Bno055<I>,
    sender: PostcardSender<Tx>,
) -> !
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
    Tx: WireTx,
{
    let mut device = Bno055Device::init(bno055).await.unwrap();
    let mut seq = 0_u32;

    loop {
        match device.parse_new_message() {
            Ok(msg) => {
                info!("IMU Message {:#?}", Debug2Format(&msg));

                if sender.publish::<ImuTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    error!("Failed to publish IMU message");
                }
            },
            Err(e) => error!("Failed to read BNO055: {:?}", Debug2Format(&e)),
        }
    }
}
