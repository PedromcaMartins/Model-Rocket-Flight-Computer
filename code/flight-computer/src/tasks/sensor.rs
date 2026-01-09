use defmt_or_log::{debug, error};
use embassy_futures::join::join;

use crate::{core::trace::TraceAsync, interfaces::SensorDevice, sync::broadcast_record};

#[inline]
pub async fn sensor_task<S>(mut sensor: S) -> !
where
    S: SensorDevice,
{
    let mut sensor_ticker = sensor.ticker();

    loop {
        let mut trace = TraceAsync::start(S::NAME);

        trace.before_await();
        let res = join(
            sensor_ticker.next(),
            sensor.parse_new_data(),
        ).await;
        trace.after_await();

        match res.1 {
            Ok(msg) => {
                debug!("{}: Parsed new data", S::NAME);

                broadcast_record(msg.into());
            },
            Err(_) => error!("{}: Failed to parse data", S::NAME),
        }
    }
}
