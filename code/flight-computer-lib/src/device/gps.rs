use chrono::Timelike;
use embassy_time::Instant;
use nmea::{Nmea, SentenceType, SENTENCE_MAX_LEN};
use telemetry_messages::{FixTypeWraper, GpsMessage};
use uom::si::{length::meter, quantities::{Length, Time}, time::{hour, microsecond, minute, second}};

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum GpsError {
    #[error("NMEA parser initialization error")]
    NmeaParserInit,
    #[error("Uart read error")]
    UartRead,
    #[error("UTF-8 decoding error: Invalid characters")]
    InvalidUtf8,
    #[error("Message does not contain initial NMEA characters ('$')")]
    MessageDoesNotContainInitialNmea,
    #[error("NMEA parser error")]
    NmeaParser,
    #[error("GPS Sentence type not implemented: {0}")]
    UnimplementedSentenceType(SentenceType),
    #[error("GPS message: required fields are missing")]
    MissingFields,
}

pub struct GpsDevice<U>
where
    U: embedded_io_async::Read,
{
    uart: U,
    buf: [u8; SENTENCE_MAX_LEN],
    nmea: Nmea,
}

impl<U> GpsDevice<U>
where
    U: embedded_io_async::Read,
{
    pub fn init(uart: U) -> Result<Self, GpsError> {
        let nmea = Nmea::create_for_navigation(&[SentenceType::GGA])
            .map_err(|_| GpsError::NmeaParserInit)?;
    
        Ok(Self { 
            uart,
            buf: [0; SENTENCE_MAX_LEN],
            nmea,
        })
    }

    pub async fn parse_new_message(&mut self) -> Result<GpsMessage, GpsError> {
        self.buf.fill(0);

        let len = self.uart
            .read(&mut self.buf)
            .await
            .map_err(|_| GpsError::UartRead)?;

        let raw = core::str::from_utf8(&self.buf[..len])
            .map_err(|_| GpsError::InvalidUtf8)?;

        let raw_aligned = raw.find('$')
            .and_then(|start| raw.get(start..))
            .ok_or(GpsError::MessageDoesNotContainInitialNmea)?;

        let sentence_type = self.nmea
            .parse(raw_aligned)
            .map_err(|_| GpsError::NmeaParser)?;

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
                .ok_or(GpsError::MissingFields)?,
            longitude: self.nmea
                .longitude()
                .ok_or(GpsError::MissingFields)?,
            altitude: self.nmea
                .altitude()
                .map(Length::new::<meter>)
                .ok_or(GpsError::MissingFields)?,
            fix_time,
            fix_type: self.nmea
                .fix_type()
                .map(FixTypeWraper::new)
                .ok_or(GpsError::MissingFields)?,
            #[allow(clippy::cast_possible_truncation)]
            num_of_fix_satellites: self.nmea
                .fix_satellites()
                .ok_or(GpsError::MissingFields)? as u8,
            timestamp: Time::new::<microsecond>(Instant::now().as_micros() as f64),
        })
    }
}
