use chrono::Timelike;
use defmt_or_log::{error, info, Debug2Format};
use embassy_time::{Instant, Timer};
use nmea::{Nmea, SentenceType, SENTENCE_MAX_LEN};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{FixTypeWraper, GpsMessage, GpsTopic};
use uom::si::{angle::degree, length::meter, quantities::{Angle, Length, Time}, time::{hour, microsecond, minute, second}};

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum GpsError<IO>
where
    IO: embedded_io_async::Error,
{
    #[error("NMEA parser initialization error")]
    NmeaParserInit,
    #[error("I/O error: {0}")]
    Io(#[from] IO),
    #[error("UTF-8 decoding error")]
    Utf8Decoding,
    #[error("NMEA parsing error")]
    NmeaParsing,
    #[error("GPS Sentence type not implemented: {0}")]
    UnimplementedSentenceType(SentenceType),
    #[error("GPS message: required fields are missing")]
    MissingFields,
}

#[inline]
pub async fn gps_task<U, Tx>(
    uart: U,
    sender: PostcardSender<Tx>,
) -> !
where
    U: embedded_io_async::Read,
    Tx: WireTx,
{
    let mut parser = GpsParser::init(uart).unwrap();
    let mut seq = 0_u32;

    loop {
        match parser.parse_new_message().await {
            Ok(msg) => {
                info!("GPS Message: {:?}", Debug2Format(&msg));

                if sender.publish::<GpsTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    error!("Failed to publish GPS message");
                }
            }, 
            Err(e) => error!("Failed to read GPS: {:?}", Debug2Format(&e)),
        }

        Timer::after_millis(100).await;
    }
}

struct GpsParser<U>
where
    U: embedded_io_async::Read,
{
    uart: U,
    buf: [u8; SENTENCE_MAX_LEN],
    nmea: Nmea,
}

impl<U> GpsParser<U>
where
    U: embedded_io_async::Read,
{
    pub fn init(uart: U) -> Result<Self, GpsError<U::Error>> {
        let nmea = Nmea::create_for_navigation(&[SentenceType::GGA])
            .map_err(|_| GpsError::NmeaParserInit)?;
    
        Ok(Self { 
            uart,
            buf: [0; SENTENCE_MAX_LEN],
            nmea,
        })
    }

    pub async fn parse_new_message(&mut self) -> Result<GpsMessage, GpsError<U::Error>> {
        let len = self.uart
            .read(&mut self.buf)
            .await
            .map_err(GpsError::Io)?;

        let raw = core::str::from_utf8(&self.buf[..len])
            .map_err(|_| GpsError::Utf8Decoding)?;

        let sentence_type = self.nmea
            .parse(raw)
            .map_err(|_| GpsError::NmeaParsing)?;

        if sentence_type != SentenceType::GGA {
            return Err(GpsError::UnimplementedSentenceType(sentence_type));
        }

        let fix_time = self.nmea.fix_time.ok_or(GpsError::MissingFields).map(|t| {
            Time::new::<hour>(t.hour().into())
                + Time::new::<minute>(t.minute().into())
                + Time::new::<second>(t.second().into())
        })?;

        Ok(GpsMessage {
            latitude: self.nmea
                .latitude()
                .map(Angle::new::<degree>)
                .ok_or_else(|| GpsError::MissingFields)?,
            longitude: self.nmea
                .longitude()
                .map(Angle::new::<degree>)
                .ok_or_else(|| GpsError::MissingFields)?,
            altitude: self.nmea
                .altitude()
                .map(Length::new::<meter>)
                .ok_or_else(|| GpsError::MissingFields)?,
            fix_time,
            fix_type: self.nmea
                .fix_type()
                .map(FixTypeWraper::new)
                .ok_or_else(|| GpsError::MissingFields)?,
            #[allow(clippy::cast_possible_truncation)]
            num_of_fix_satellites: self.nmea
                .fix_satellites()
                .ok_or(GpsError::MissingFields)? as u8,
            timestamp: Time::new::<microsecond>(Instant::now().as_micros()),
        })
    }
}
