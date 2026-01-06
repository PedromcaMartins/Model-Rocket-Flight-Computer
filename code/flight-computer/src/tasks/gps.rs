use core::num::Wrapping;

use embassy_time::Ticker;
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use proto::{GpsMessage, GpsTopic};
use defmt_or_log::{debug, error, warn};

use crate::{config::DataAcquisitionConfig, core::trace::TraceAsync, interfaces::SensorDevice, sync::GPS_SD_CARD_CHANNEL};

#[inline]
pub async fn gps_task<
    S, Tx,
> (
    mut gps: S,
    postcard_sender: &PostcardSender<Tx>,
) -> !
where
    S: SensorDevice<DataMessage = GpsMessage>,
    Tx: WireTx,
{
    let sd_card_sender = GPS_SD_CARD_CHANNEL.sender();
    let mut seq: Wrapping<u32> = Wrapping::default();

    let mut sensor_ticker = Ticker::every(DataAcquisitionConfig::GPS_TICKER_PERIOD);

    loop {
        let mut trace = TraceAsync::start("gps_task_loop");

        trace.before_await();
        sensor_ticker.next().await;
        let res = gps.parse_new_message().await;
        trace.after_await();

        match res {
            Ok(msg) => {
                debug!("GPS: Parsed new message");

                if postcard_sender.publish::<GpsTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                    seq += 1;
                } else { 
                    warn!("GPS: Failed to publish to Postcard");
                }

                if sd_card_sender.try_send(msg).is_err() {
                    error!("GPS: Failed to send to SD card");
                }
            }, 
            Err(_) => error!("GPS: Failed to parse message"),
        }
    }
}
