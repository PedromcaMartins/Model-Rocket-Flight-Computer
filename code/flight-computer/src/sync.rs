use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal, watch::Watch};
use proto::{AltimeterMessage, Altitude, FlightState, GpsMessage, ImuMessage};
use crate::config::TasksConfig;

pub static LATEST_ALTITUDE_SIGNAL: Signal<CriticalSectionRawMutex, Altitude> = Signal::new();

pub static FLIGHT_STATE_WATCH: Watch<CriticalSectionRawMutex, FlightState, { TasksConfig::FLIGHT_STATE_WATCH_CONSUMERS }> = Watch::new();

pub static ALTIMETER_SD_CARD_CHANNEL: Channel<CriticalSectionRawMutex, AltimeterMessage, { TasksConfig::ALTIMETER_SD_CARD_CHANNEL_DEPTH }> = Channel::new();
pub static GPS_SD_CARD_CHANNEL:       Channel<CriticalSectionRawMutex, GpsMessage, { TasksConfig::GPS_SD_CARD_CHANNEL_DEPTH }> = Channel::new();
pub static IMU_SD_CARD_CHANNEL:       Channel<CriticalSectionRawMutex, ImuMessage, { TasksConfig::IMU_SD_CARD_CHANNEL_DEPTH }> = Channel::new();
