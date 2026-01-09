use defmt_or_log::warn;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal, watch::Watch};
use proto::{Record, RecordData, sensor_data::Altitude};
use crate::config::TasksConfig;

pub static LATEST_ALTITUDE_SIGNAL: Signal<CriticalSectionRawMutex, Altitude> = Signal::new();

pub static FLIGHT_STATE_WATCH: Watch<CriticalSectionRawMutex, Record, { TasksConfig::FLIGHT_STATE_WATCH_CONSUMERS }> = Watch::new();

pub static ALTIMETER_DATA_TO_GROUNDSTATION_SIGNAL: Signal<CriticalSectionRawMutex, Record> = Signal::new();
pub static GPS_DATA_TO_GROUNDSTATION_SIGNAL: Signal<CriticalSectionRawMutex, Record> = Signal::new();
pub static IMU_DATA_TO_GROUNDSTATION_SIGNAL: Signal<CriticalSectionRawMutex, Record> = Signal::new();

pub static RECORD_TO_STORAGE_CHANNEL: Channel<CriticalSectionRawMutex, Record, { TasksConfig::RECORD_TO_STORAGE_CHANNEL_DEPTH }> = Channel::new();

pub fn broadcast_record(record: Record) {
    // groundstation is picky about records
    match record.payload() {
        RecordData::Gps(_) =>           GPS_DATA_TO_GROUNDSTATION_SIGNAL.signal(record.clone()),
        RecordData::Imu(_) =>           IMU_DATA_TO_GROUNDSTATION_SIGNAL.signal(record.clone()),
        RecordData::FlightState(_) =>   FLIGHT_STATE_WATCH.sender().send(record.clone()),
        RecordData::Altimeter(payload) => {
            LATEST_ALTITUDE_SIGNAL.signal(payload.altitude);
            ALTIMETER_DATA_TO_GROUNDSTATION_SIGNAL.signal(record.clone());
        },
        RecordData::Event(_) | RecordData::Error(_) => (), // Errors and Events are not broadcast to the ground station
    }

    // storage consumes all record
    if let Err(e) = RECORD_TO_STORAGE_CHANNEL.try_send(record) {
        // Log or handle the error gracefully
        warn!("Failed to send record to storage channel: {:?}", e);
    }
}
