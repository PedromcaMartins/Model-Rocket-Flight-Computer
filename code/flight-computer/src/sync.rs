use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal, watch::Watch};
use proto::{Record, flight_state::FlightState, sensor_data::Altitude};
use crate::config::TasksConfig;

pub static LATEST_ALTITUDE_SIGNAL: Signal<CriticalSectionRawMutex, Altitude> = Signal::new();

pub static FLIGHT_STATE_WATCH: Watch<CriticalSectionRawMutex, FlightState, { TasksConfig::FLIGHT_STATE_WATCH_CONSUMERS }> = Watch::new();
pub static RECORD_TO_STORAGE_CHANNEL: Channel<CriticalSectionRawMutex, Record, { TasksConfig::RECORD_TO_STORAGE_CHANNEL_DEPTH }> = Channel::new();
