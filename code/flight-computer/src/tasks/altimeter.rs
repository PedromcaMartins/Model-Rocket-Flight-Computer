use core::num::Wrapping;

use defmt_or_log::{debug, error, warn};
use embassy_time::Ticker;
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use proto::{RecordTopic, sensor_data::AltimeterData};

use crate::{config::DataAcquisitionConfig, core::trace::TraceAsync, interfaces::SensorDevice, sync::{RECORD_TO_STORAGE_CHANNEL, LATEST_ALTITUDE_SIGNAL}};

#[inline]
pub async fn altimeter_task<S, Tx> (mut altimeter: S, postcard_sender: &PostcardSender<Tx>) -> !
where
    S: SensorDevice<Data = AltimeterData>,
    Tx: WireTx,
{
    let storage_sender = RECORD_TO_STORAGE_CHANNEL.sender();
    let mut seq: Wrapping<u32> = Wrapping::default();

    let mut sensor_ticker = Ticker::every(DataAcquisitionConfig::ALTIMETER_TICKER_PERIOD);

    loop {
        let mut trace = TraceAsync::start("altimeter_task_loop");

        trace.before_await();
        sensor_ticker.next().await;
        let res = altimeter.parse_new_data().await;
        trace.after_await();

        match res {
            Ok(msg) => {
                debug!("Altimeter: Parsed new data");

                LATEST_ALTITUDE_SIGNAL.signal(msg.altitude);

                let msg = msg.into();
                if postcard_sender.publish::<RecordTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                    seq += 1;
                } else { 
                    warn!("Altimeter: Failed to publish to Postcard");
                }

                if storage_sender.try_send(msg).is_err() {
                    error!("Altimeter: Failed to send to Storage");
                }
            },
            Err(_) => error!("Altimeter: Failed to parse data"),
        }
    }
}
