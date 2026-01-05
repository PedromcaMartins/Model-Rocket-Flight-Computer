use core::num::Wrapping;

use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender};
use embassy_time::Ticker;
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use proto::{ImuMessage, ImuTopic};
use defmt_or_log::{debug, error, warn};

use crate::{config::DataAcquisitionConfig, interfaces::SensorDevice, core::trace::TraceAsync};

#[inline]
pub async fn imu_task<
    S, M, Tx,
    const DEPTH_DATA: usize,
> (
    mut imu: S,
    config: DataAcquisitionConfig,
    sd_card_sender: Sender<'static, M, ImuMessage, DEPTH_DATA>,
    postcard_sender: PostcardSender<Tx>,
) -> !
where
    S: SensorDevice<DataMessage = ImuMessage>,
    M: RawMutex + 'static,
    Tx: WireTx,
{
    let mut seq: Wrapping<u32> = Wrapping::default();

    let mut sensor_ticker = Ticker::every(config.imu_ticker_period);

    loop {
        let mut trace = TraceAsync::start("imu_task_loop");

        trace.before_await();
        sensor_ticker.next().await;
        let res = imu.parse_new_message().await;
        trace.after_await();

        match res {
            Ok(msg) => {
                debug!("IMU: Parsed new message");

                if postcard_sender.publish::<ImuTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                    seq += 1;
                } else { 
                    warn!("IMU: Failed to publish to Postcard");
                }

                if sd_card_sender.try_send(msg).is_err() {
                    error!("IMU: Failed to send to SD card");
                }
            },
            Err(_) => error!("IMU: Failed to parse message"),
        }
    }
}
