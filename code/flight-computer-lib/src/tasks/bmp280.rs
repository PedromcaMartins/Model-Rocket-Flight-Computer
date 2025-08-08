use core::{convert::Infallible, fmt::Debug};

use bmp280_ehal::BMP280;
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::Timer;
use embedded_hal::i2c::{I2c, SevenBitAddress};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{AltimeterMessage, AltimeterTopic};
use uom::si::f64;

use crate::device::bmp280::Bmp280Device;

#[derive(Debug, Clone, Default)]
pub struct AltimeterSystemStatus {
    pub message_parsed: u64,
    pub failed_to_initialize_device: bool,
    pub failed_to_parse_message: u64,
    pub failed_to_publish_to_postcard: u64,
    pub failed_to_publish_to_sd_card: u64,
}

#[inline]
pub async fn bmp280_task<
    I, E, M, Tx, 
    const DEPTH: usize,
> (
    bmp280: BMP280<I>,
    altitude_signal: &'static Signal<M, f64::Length>,
    sd_card_sender: embassy_sync::channel::Sender<'static, M, AltimeterMessage, DEPTH>,
    postcard_sender: PostcardSender<Tx>,
) -> Result<Infallible, ()>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut status = AltimeterSystemStatus::default();

    let Ok(mut device) = Bmp280Device::init(bmp280) else {
        status.failed_to_initialize_device = true;
        return Err(());
    };
    let mut seq = 0_u32;

    loop {
        match device.parse_new_message() {
            Err(_) => status.failed_to_parse_message += 1,
            Ok(msg) => {
                status.message_parsed += 1;

                if postcard_sender.publish::<AltimeterTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    status.failed_to_publish_to_postcard += 1;
                }

                altitude_signal.signal(msg.altitude);

                if sd_card_sender.try_send(msg).is_err() {
                    status.failed_to_publish_to_sd_card += 1;
                }
            },
        }

        Timer::after_millis(50).await;
    }
}
