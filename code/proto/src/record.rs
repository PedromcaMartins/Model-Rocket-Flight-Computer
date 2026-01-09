use derive_more::From;

use crate::{Serialize, Deserialize, Schema, error::Error, event::Event, flight_state::FlightState, sensor_data::{AltimeterData, GpsData, ImuData}};

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
    timestamp: u64,
    /// Unique ID.
    uid: u32,
    /// The recorded data.
    payload: RecordData,
}

#[allow(clippy::must_use_candidate)]
impl Record {
    pub const fn timestamp_ticks(&self) -> u64 {
        self.timestamp
    }

    pub const fn uid(&self) -> u32 {
        self.uid
    }

    pub const fn payload(&self) -> &RecordData {
        &self.payload
    }

    pub const fn into_inner(self) -> (u64, u32, RecordData) {
        (self.timestamp, self.uid, self.payload)
    }
}

#[cfg(feature = "embassy-time")]
mod impls {
    use core::sync::atomic::{AtomicU32, Ordering};

    use embassy_time::Instant;

    use super::{Record, RecordData, AltimeterData, GpsData, ImuData, FlightState, Event, Error};

    static UID_COUNTER: AtomicU32 = AtomicU32::new(0);

    impl Record {
        fn new_id() -> u32 {
            UID_COUNTER.fetch_add(1, Ordering::SeqCst)
        }

        fn current_timestamp() -> u64 {
            Instant::now().as_ticks()
        }
    }

    impl From<AltimeterData> for Record {
        fn from(value: AltimeterData) -> Self {
            Self {
                timestamp: Self::current_timestamp(),
                uid: Self::new_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<GpsData> for Record {
        fn from(value: GpsData) -> Self {
            Self {
                timestamp: Self::current_timestamp(),
                uid: Self::new_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<ImuData> for Record {
        fn from(value: ImuData) -> Self {
            Self {
                timestamp: Self::current_timestamp(),
                uid: Self::new_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<FlightState> for Record {
        fn from(value: FlightState) -> Self {
            Self {
                timestamp: Self::current_timestamp(),
                uid: Self::new_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<Event> for Record {
        fn from(value: Event) -> Self {
            Self {
                timestamp: Self::current_timestamp(),
                uid: Self::new_id(),
                payload: RecordData::from(value),
            }
        }
    }

    impl From<Error> for Record {
        fn from(value: Error) -> Self {
            Self {
                timestamp: Self::current_timestamp(),
                uid: Self::new_id(),
                payload: RecordData::from(value),
            }
        }
    }
}
