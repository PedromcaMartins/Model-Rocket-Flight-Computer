use defmt::{error, info, Debug2Format};
use embassy_time::Timer;
use nmea::{Nmea, SentenceType, SENTENCE_MAX_LEN};

#[inline]
pub async fn gps_task<U>(mut uart: U) 
where
    U: embedded_io_async::Read
{
    let mut buf = [0; SENTENCE_MAX_LEN];
    let mut nmea = Nmea::create_for_navigation(&[SentenceType::GGA]).unwrap();

    loop {
        if let Ok(len) = uart.read(&mut buf).await {
            if let Ok(message) = core::str::from_utf8(&buf[..len]) {
                match nmea.parse(message) {
                    Ok(_) => info!("GPS: {:?}", Debug2Format(&nmea)),
                    Err(e) => error!("ErroDebug2Format(&r): {:?}, Message: {}", Debug2Format(&e), message),
                }
            }
        }

        Timer::after_millis(100).await;
    }
}
