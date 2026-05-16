use crate::log::{debug, error};
use embassy_futures::join::join;
use embassy_time::with_timeout;

use crate::{interfaces::{Led, Sensor}, sync::broadcast_record};

#[inline]
pub async fn sensor_task<S, LED>(mut sensor: S, mut led: LED) -> !
where
    S: Sensor,
    LED: Led,
{
    let mut sensor_ticker = sensor.ticker();

    loop {
        let timeout = S::TICK_INTERVAL + S::TICK_INTERVAL / 2;
        let ((), data) = join(
            sensor_ticker.next(),
            with_timeout(timeout, sensor.parse_new_data()),
        ).await;

        led.off().await.unwrap_or_else(|e| error!("{}: Status Led error: {:?}", S::NAME, e));

        match data {
            Err(_) => error!("{}: Timed out reading sensor data", S::NAME),
            Ok(Err(_)) => error!("{}: Failed to parse data", S::NAME),
            Ok(Ok(msg)) => {
                debug!("{}: Parsed new data", S::NAME);
                led.on().await.unwrap_or_else(|e| error!("{}: Status Led error: {:?}", S::NAME, e));

                broadcast_record(msg.into());
            },
        }
    }
}
