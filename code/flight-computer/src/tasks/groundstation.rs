use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

use defmt_or_log::Debug2Format;
use defmt_or_log::warn;

use embassy_futures::select::Either;
use embassy_futures::select::select;
use embassy_time::Ticker;
use postcard_rpc::header::VarSeq;
use postcard_rpc::server::AsWireTxErrorKind;
use postcard_rpc::server::{Sender as PostcardSender, WireTx};
use proto::Record;
use proto::RecordTopic;

use crate::config::GroundStationConfig;
use crate::interfaces::Led;
use crate::sync::ALTIMETER_DATA_TO_GROUNDSTATION_SIGNAL;
use crate::sync::FLIGHT_STATE_WATCH;
use crate::sync::GPS_DATA_TO_GROUNDSTATION_SIGNAL;
use crate::sync::IMU_DATA_TO_GROUNDSTATION_SIGNAL;

static UID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[inline]
async fn send_to_ground_station<Tx>(postcard_sender: &PostcardSender<Tx>, msg: &Record)
where
    Tx: WireTx,
{
    if let Err(err) = postcard_sender.publish::<RecordTopic>(
        VarSeq::Seq4(
            UID_COUNTER.fetch_add(1, Ordering::Relaxed)
        ), 
        msg
    ).await {
        warn!("GroundStation: Failed to send record to ground station: {:?}", Debug2Format(&err.as_kind()));
    }
}

#[inline]
pub async fn groundstation_task<Tx, LED>(postcard_sender: &PostcardSender<Tx>, mut led: LED) -> !
where
    Tx: WireTx,
    LED: Led,
{
    let mut flight_state_receiver = FLIGHT_STATE_WATCH.receiver().expect("Not enough flight state consumers");

    let mut sensor_data_ticker = Ticker::every(GroundStationConfig::SEND_SENSOR_DATA_TICK_INTERVAL);

    loop {
        let result = select(
            flight_state_receiver.changed(),
            sensor_data_ticker.next(),
        ).await;

        if led.on().await.is_err() { warn!("GroundStation: Status Led error"); }

        match result {
            Either::First(state) => {
                send_to_ground_station(postcard_sender, &state).await;
            },
            Either::Second(()) => {
                for signal in [
                    &GPS_DATA_TO_GROUNDSTATION_SIGNAL,
                    &IMU_DATA_TO_GROUNDSTATION_SIGNAL,
                    &ALTIMETER_DATA_TO_GROUNDSTATION_SIGNAL,
                ] {
                    if let Some(record) = signal.try_take() {
                        send_to_ground_station(postcard_sender, &record).await;
                    }
                }
            },
        }

        if led.off().await.is_err() { warn!("GroundStation: Status Led error"); }
    }
}
