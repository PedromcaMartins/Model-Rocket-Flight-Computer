use crate::log::{debug, error};
use embassy_futures::join::join;

use crate::{interfaces::{Led, Sensor}, sync::broadcast_record};

#[inline]
pub async fn sensor_task<S, LED>(mut sensor: S, mut led: LED) -> !
where
    S: Sensor,
    LED: Led,
{
    let mut sensor_ticker = sensor.ticker();

    loop {
        let res = join(
            sensor_ticker.next(),
            sensor.parse_new_data(),
        ).await;

        led.off().await.unwrap_or_else(|e| error!("{}: Status Led error: {:?}", S::NAME, e));

        match res.1 {
            Ok(msg) => {
                debug!("{}: Parsed new data", S::NAME);
                led.on().await.unwrap_or_else(|e| error!("{}: Status Led error: {:?}", S::NAME, e));

                broadcast_record(msg.into());
            },
            Err(_) => error!("{}: Failed to parse data", S::NAME),
        }
    }
}
