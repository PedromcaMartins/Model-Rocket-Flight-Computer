use core::{convert::Infallible, fmt::Debug, num::Wrapping};

use bmp280_ehal::BMP280;
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::i2c::{I2c, SevenBitAddress};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{AltimeterMessage, AltimeterTopic};
use uom::si::f64;

use crate::{device::bmp280::Bmp280Device, model::system_status::AltimeterSystemStatus};

#[inline]
pub async fn bmp280_task<
    I, E, M, Tx, 
    const DEPTH: usize,
> (
    bmp280: BMP280<I>,
    altitude_signal: &'static Signal<M, f64::Length>,
    status_signal: &'static Signal<M, AltimeterSystemStatus>,
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
        
                        if postcard_sender.publish::<AltimeterTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                            seq += 1;
                        } else {
                            status.failed_to_publish_to_postcard += 1;
                        }
        
                        altitude_signal.signal(msg.altitude);
        
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
