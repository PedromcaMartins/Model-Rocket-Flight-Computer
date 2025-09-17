use core::num::Wrapping;

use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender};
use embassy_time::{Duration, Ticker};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{GpsMessage, GpsTopic};
use defmt_or_log::{debug, error};

use crate::model::sensor_device::SensorDevice;

#[inline]
pub async fn gps_task<
    S, M, Tx,
    const DEPTH_DATA: usize,
> (
    mut gps: S,
    sd_card_sender: Sender<'static, M, GpsMessage, DEPTH_DATA>,
    postcard_sender: PostcardSender<Tx>,
) -> !
where
    S: SensorDevice<DataMessage = GpsMessage>,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut seq: Wrapping<u32> = Wrapping::default();

    let mut sensor_ticker = Ticker::every(Duration::from_millis(50));

    loop {
        sensor_ticker.next().await;

        match gps.parse_new_message().await {
            Ok(msg) => {
                debug!("GPS: Parsed new message");

                if postcard_sender.publish::<GpsTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                    seq += 1;
                } else { 
                    error!("GPS: Failed to publish to Postcard");
                }

                if sd_card_sender.try_send(msg).is_err() {
                    error!("GPS: Failed to send to SD card");
                }
            }, 
            Err(_) => error!("GPS: Failed to parse message"),
        }
    }
}
