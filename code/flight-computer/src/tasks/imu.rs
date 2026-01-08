use core::num::Wrapping;

use embassy_time::Ticker;
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use proto::{RecordTopic, sensor_data::ImuData};
use defmt_or_log::{debug, error, warn};

use crate::{config::DataAcquisitionConfig, core::trace::TraceAsync, interfaces::SensorDevice, sync::RECORD_TO_STORAGE_CHANNEL};

#[inline]
pub async fn imu_task<S, Tx> (mut imu: S, postcard_sender: &PostcardSender<Tx>) -> !
where
    S: SensorDevice<Data = ImuData>,
    Tx: WireTx,
{
    let storage_sender = RECORD_TO_STORAGE_CHANNEL.sender();
    let mut seq: Wrapping<u32> = Wrapping::default();

    let mut sensor_ticker = Ticker::every(DataAcquisitionConfig::IMU_TICKER_PERIOD);

    loop {
        let mut trace = TraceAsync::start("imu_task_loop");

        trace.before_await();
        sensor_ticker.next().await;
        let res = imu.parse_new_data().await;
        trace.after_await();

        match res {
            Ok(msg) => {
                debug!("IMU: Parsed new data");

                let msg = msg.into();
                if postcard_sender.publish::<RecordTopic>(VarSeq::Seq4(seq.0), &msg).await.is_ok() {
                    seq += 1;
                } else { 
                    warn!("IMU: Failed to publish to Postcard");
                }

                if storage_sender.try_send(msg).is_err() {
                    error!("IMU: Failed to send to Storage");
                }
            },
            Err(_) => error!("IMU: Failed to parse data"),
        }
    }
}
