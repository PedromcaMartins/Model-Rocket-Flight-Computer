use nmea::{Nmea, SentenceType, SENTENCE_MAX_LEN};
use static_cell::ConstStaticCell;
use proto::sensor_data::{Altitude, GpsCoordinates, GpsData};
use proto::uom::si::length::meter;

use crate::config::DataAcquisitionConfig;
use crate::interfaces::Sensor;

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

impl<U> Sensor for GpsDevice<U>
where
    U: embedded_io_async::Read,
{
    type Data = GpsData;
    type Error = GpsError;

    const NAME: &'static str = "GPS";
    const TICK_INTERVAL: embassy_time::Duration = DataAcquisitionConfig::GPS_TICK_INTERVAL;

    #[allow(clippy::cast_possible_truncation)]
    async fn parse_new_data(&mut self) -> Result<Self::Data, Self::Error> {
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

        Ok(GpsData {
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
            fix_time: self.nmea
                .fix_time
                .ok_or(GpsError::MissingFields)?
                .into(),
            fix_type: self.nmea
                .fix_type()
                .ok_or(GpsError::MissingFields)?
                .into(),
            num_of_fix_satellites: self.nmea
                .fix_satellites()
                .ok_or(GpsError::MissingFields)?
                as u8,
        })
    }
}
