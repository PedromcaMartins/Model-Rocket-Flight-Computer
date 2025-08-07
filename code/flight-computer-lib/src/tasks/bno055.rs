use core::fmt::Debug;

use defmt_or_log::error;
use bno055::Bno055;
use defmt_or_log::Debug2Format;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::Timer;
use embedded_hal::i2c::{I2c, SevenBitAddress};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{ImuMessage, ImuTopic};

use crate::device::bno055::Bno055Device;

#[inline]
pub async fn bno055_task<
    I, E, M, Tx,
    const DEPTH: usize,
> (
    bno055: Bno055<I>,
    sd_card_sender: embassy_sync::channel::Sender<'static, M, ImuMessage, DEPTH>,
    postcard_sender: PostcardSender<Tx>,
) -> !
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut device = Bno055Device::init(bno055).await.unwrap();
    let mut seq = 0_u32;

    loop {
        match device.parse_new_message() {
            Ok(msg) => {
                // info!("IMU Message {:#?}", Debug2Format(&msg));

                if postcard_sender.publish::<ImuTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    // error!("Failed to publish IMU message to postcard client");
                }

                if sd_card_sender.try_send(msg).is_err() {
                    // error!("Failed to send IMU message to SD card task");
                }
            },
            // Err(e) => error!("Failed to read BNO055: {:?}", Debug2Format(&e)),
            Err(_) => (),
        }

        Timer::after_millis(50).await;
    }
}
