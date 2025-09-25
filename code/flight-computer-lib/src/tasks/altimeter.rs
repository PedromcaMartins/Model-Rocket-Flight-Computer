use core::num::Wrapping;

use defmt_or_log::{debug, error, warn};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender, signal::Signal};
use embassy_time::{Duration, Ticker};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{AltimeterMessage, AltimeterTopic, Altitude};

use crate::interfaces::SensorDevice;

#[inline]
pub async fn altimeter_task<
    S, M, Tx, 
    const DEPTH_DATA: usize,
> (
    mut altimeter: S,
    latest_altitude_signal: &'static Signal<M, Altitude>,
    sd_card_sender: Sender<'static, M, AltimeterMessage, DEPTH_DATA>,
    postcard_sender: PostcardSender<Tx>,
) -> !
where
    S: SensorDevice<DataMessage = AltimeterMessage>,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut seq: Wrapping<u32> = Wrapping::default();

    let mut sensor_ticker = Ticker::every(Duration::from_millis(50));

    loop {
        sensor_ticker.next().await;

        match altimeter.parse_new_message().await {
            Ok(msg) => {
                debug!("Altimeter: Parsed new message");

                if postcard_sender.publish::<AltimeterTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                    seq += 1;
                } else { 
                    warn!("Altimeter: Failed to publish to Postcard");
                }

                latest_altitude_signal.signal(msg.altitude);

                if sd_card_sender.try_send(msg).is_err() {
                    error!("Altimeter: Failed to send to SD card");
                }
            },
            Err(_) => error!("Altimeter: Failed to parse message"),
        }
    }
}
