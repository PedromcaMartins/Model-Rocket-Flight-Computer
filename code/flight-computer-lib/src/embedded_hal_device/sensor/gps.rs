use chrono::Timelike;
use embassy_time::Instant;
use nmea::{Nmea, SentenceType, SENTENCE_MAX_LEN};
use static_cell::ConstStaticCell;
use telemetry_messages::{FixTypeWrapper, GpsCoordinates, GpsMessage, Timestamp, Altitude};
use uom::si::length::meter;

use crate::model::sensor_device::SensorDevice;

#[defmt_or_log::maybe_derive_format]
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
    buf: &'static mut [u8],
    nmea: Nmea,
}

impl<U> GpsDevice<U>
where
    U: embedded_io_async::Read,
{
    pub fn init(uart: U) -> Result<Self, GpsError> {
        static BUFFER: ConstStaticCell<[u8; SENTENCE_MAX_LEN]> = ConstStaticCell::new([0_u8; SENTENCE_MAX_LEN]);
        let buf = BUFFER.take();

        let nmea = Nmea::create_for_navigation(&[SentenceType::GGA])
            .map_err(|_| GpsError::NmeaParserInit)?;

        Ok(Self { 
            uart,
            buf,
            nmea,
        })
    }
}

impl<U> SensorDevice for GpsDevice<U>
where
    U: embedded_io_async::Read,
{
    type DataMessage = GpsMessage;
    type DeviceError = GpsError;

    #[allow(clippy::cast_possible_truncation)]
    async fn parse_new_message(&mut self) -> Result<Self::DataMessage, Self::DeviceError> {
        self.buf.fill(0);

        let len = self.uart
            .read(self.buf)
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
            Timestamp {
                hour: t.hour() as u8,
                minute: t.minute() as u8,
                second: t.second() as u8,
            }
        })?;

        Ok(GpsMessage {
            coordinates: GpsCoordinates {
                latitude: self.nmea
                    .latitude()
                    .map(|l| l as f32)
                    .ok_or(GpsError::MissingFields)?,
                longitude: self.nmea
                    .longitude()
                    .map(|l| l as f32)
                    .ok_or(GpsError::MissingFields)?,
            },
            altitude: self.nmea
                .altitude()
                .map(Altitude::new::<meter>)
                .ok_or(GpsError::MissingFields)?,
            fix_time,
            fix_type: self.nmea
                .fix_type()
                .map(FixTypeWrapper::new)
                .ok_or(GpsError::MissingFields)?,
            num_of_fix_satellites: self.nmea
                .fix_satellites()
                .ok_or(GpsError::MissingFields)?
                as u8,
            timestamp: Instant::now().as_micros(),
        })
    }
}
