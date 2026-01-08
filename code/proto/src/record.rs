use core::sync::atomic::{AtomicU32, Ordering};

use derive_more::From;
use embassy_time::Instant;

use crate::{Serialize, Deserialize, Schema, error::Error, event::Event, flight_state::FlightState, sensor_data::{AltimeterData, GpsData, ImuData}};

static UID_COUNTER: AtomicU32 = AtomicU32::new(0);

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
    /// Timestamp in microseconds.
    timestamp: u64,
    /// Unique ID.
    uid: u32,
    /// The recorded data.
    payload: RecordData,
}

#[allow(clippy::must_use_candidate)]
impl Record {
    fn new_id() -> u32 {
        UID_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    pub const fn timestamp_ms(&self) -> u64 {
        self.timestamp
    }

    pub const fn uid(&self) -> u32 {
        self.uid
    }

    pub const fn into_payload(self) -> RecordData {
        self.payload
    }

    pub const fn into_inner(self) -> (u64, u32, RecordData) {
        (self.timestamp, self.uid, self.payload)
    }
}

impl From<AltimeterData> for Record {
    fn from(value: AltimeterData) -> Self {
        Self {
            timestamp: Instant::now().as_micros(),
            uid: Self::new_id(),
            payload: RecordData::from(value),
        }
    }
}

impl From<GpsData> for Record {
    fn from(value: GpsData) -> Self {
        Self {
            timestamp: Instant::now().as_micros(),
            uid: Self::new_id(),
            payload: RecordData::from(value),
        }
    }
}

impl From<ImuData> for Record {
    fn from(value: ImuData) -> Self {
        Self {
            timestamp: Instant::now().as_micros(),
            uid: Self::new_id(),
            payload: RecordData::from(value),
        }
    }
}

impl From<FlightState> for Record {
    fn from(value: FlightState) -> Self {
        Self {
            timestamp: Instant::now().as_micros(),
            uid: Self::new_id(),
            payload: RecordData::from(value),
        }
    }
}

impl From<Event> for Record {
    fn from(value: Event) -> Self {
        Self {
            timestamp: Instant::now().as_micros(),
            uid: Self::new_id(),
            payload: RecordData::from(value),
        }
    }
}

impl From<Error> for Record {
    fn from(value: Error) -> Self {
        Self {
            timestamp: Instant::now().as_micros(),
            uid: Self::new_id(),
            payload: RecordData::from(value),
        }
    }
}
