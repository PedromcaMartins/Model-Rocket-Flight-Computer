use core::num::Wrapping;

use defmt_or_log::{debug, error, warn};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender, signal::Signal};
use embassy_time::Ticker;
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use proto::{AltimeterMessage, AltimeterTopic, Altitude};

use crate::{config::DataAcquisitionConfig, interfaces::SensorDevice, core::trace::TraceAsync};

#[inline]
pub async fn altimeter_task<
    S, M, Tx, 
    const DEPTH_DATA: usize,
> (
    mut altimeter: S,
    config: DataAcquisitionConfig,
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

    let mut sensor_ticker = Ticker::every(config.altimeter_ticker_period);

    loop {
        let mut trace = TraceAsync::start("altimeter_task_loop");

        trace.before_await();
        sensor_ticker.next().await;
        let res = altimeter.parse_new_message().await;
        trace.after_await();

        match res {
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
