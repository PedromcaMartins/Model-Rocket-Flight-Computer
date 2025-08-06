use core::fmt::Debug;

use defmt_or_log::{info, error, Debug2Format};
use bmp280_ehal::BMP280;
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embedded_hal::i2c::{I2c, SevenBitAddress};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::AltimeterTopic;
use uom::si::f64;

use crate::device::bmp280::Bmp280Device;

#[inline]
pub async fn bmp280_task<I, E, M, Tx>(
    bmp280: BMP280<I>,
    altitude_signal: &'static Signal<M, f64::Length>,
    sender: PostcardSender<Tx>,
) -> !
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut device = Bmp280Device::init(bmp280).unwrap();
    let mut seq = 0_u32;

    loop {
        match device.parse_new_message() {
            Ok(msg) => {
                info!("Altitude Message {:#?}", Debug2Format(&msg));

                if sender.publish::<AltimeterTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    error!("Failed to publish Altimeter message");
                }

                altitude_signal.signal(msg.altitude);
            },
            Err(e) => error!("Failed to read BMP280: {:?}", Debug2Format(&e)),
        }
    }
}
