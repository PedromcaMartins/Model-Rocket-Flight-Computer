use derive_more::From;

use crate::{Serialize, Deserialize, Schema, error::Error, event::Event, flight_state::FlightState, sensor_data::{AltimeterData, GpsData, ImuData}, record::{tick_hz::Timestamp, uid::Uid}};

pub mod tick_hz;
pub mod uid;

#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq, From)]
pub enum RecordData {
    Altimeter(AltimeterData),
    Gps(GpsData),
    Imu(ImuData),
    FlightState(FlightState),
    Event(Event),
    Error(Error),
}

#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq)]
pub struct Record {
    /// Timestamp in ticks.
    timestamp: Timestamp,
    /// Unique ID.
    uid: Uid,
    /// The recorded data.
    payload: RecordData,
}

#[allow(clippy::must_use_candidate)]
impl Record {
    pub const fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub const fn uid(&self) -> Uid {
        self.uid
    }

    pub const fn payload(&self) -> &RecordData {
        &self.payload
    }

    pub const fn into_inner(self) -> (Timestamp, Uid, RecordData) {
        (self.timestamp, self.uid, self.payload)
    }
}

#[cfg(feature = "embassy-time")]
mod impls {
    use crate::record::{tick_hz::Timestamp, uid::Uid};

    use super::{Record, RecordData, AltimeterData, GpsData, ImuData, FlightState, Event, Error};

    impl From<AltimeterData> for Record {
        fn from(value: AltimeterData) -> Self {
            Self {
                timestamp: Timestamp::now(),
                uid: Uid::generate_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<GpsData> for Record {
        fn from(value: GpsData) -> Self {
            Self {
                timestamp: Timestamp::now(),
                uid: Uid::generate_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<ImuData> for Record {
        fn from(value: ImuData) -> Self {
            Self {
                timestamp: Timestamp::now(),
                uid: Uid::generate_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<FlightState> for Record {
        fn from(value: FlightState) -> Self {
            Self {
                timestamp: Timestamp::now(),
                uid: Uid::generate_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<Event> for Record {
        fn from(value: Event) -> Self {
            Self {
                timestamp: Timestamp::now(),
                uid: Uid::generate_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<Error> for Record {
        fn from(value: Error) -> Self {
            Self {
                timestamp: Timestamp::now(),
                uid: Uid::generate_id(),
                payload: RecordData::from(value),
            }
        }
    }
}
