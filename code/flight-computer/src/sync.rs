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

#[cfg(test)]
mod tests {
    use crate::test_utils::{ms, sensor_data::{random_altimeter_data, random_gps_data, random_imu_data}};

    use super::*;
    use proto::{Record, flight_state::FlightState};

    #[test_log::test(rstest::rstest)]
    #[async_std::test]
    #[serial_test::serial]
    #[case(random_altimeter_data(), ALTIMETER_DATA_TO_GROUNDSTATION_SIGNAL.wait())]
    #[case(random_gps_data(), GPS_DATA_TO_GROUNDSTATION_SIGNAL.wait())]
    #[case(random_imu_data(), IMU_DATA_TO_GROUNDSTATION_SIGNAL.wait())]
    #[case(FlightState::default(), async { let mut rec = FLIGHT_STATE_WATCH.receiver().expect("Not enough flight state consumers"); rec.changed().await })]
    #[timeout(ms(100))]
    async fn broadcast_record_to_groundstation(
        #[case] record: impl Into<Record>, 
        #[case] #[future] receiver: Record,
        #[values(TasksConfig::RECORD_TO_STORAGE_CHANNEL_DEPTH)] records_sent: usize,
    ) {
        let record = record.into();
        for _ in 0..records_sent {
            broadcast_record(record.clone());
        }

        // Check if the record was sent to groundstation
        assert_eq!(receiver.await, record);

        // Check if the record was sent to storage
        for _ in 0..records_sent {
            let received_record = RECORD_TO_STORAGE_CHANNEL.receiver().receive().await;
            assert_eq!(received_record, record);
        }
    }

    fn random_altitude() -> Altitude {
        random_altimeter_data().altitude
    }

    #[test_log::test(rstest::rstest)]
    #[async_std::test]
    #[serial_test::serial]
    #[case(1)]
    #[case(10)]
    #[case(1_000)]
    #[timeout(ms(100))]
    async fn latest_altitude_signal(#[case] updates: usize) {
        for _ in 0..=updates {
            let altitude = random_altitude();
            LATEST_ALTITUDE_SIGNAL.signal(altitude);

            let received_altitude = LATEST_ALTITUDE_SIGNAL.wait().await;
            assert_eq!(received_altitude, altitude);
        }

        assert!(LATEST_ALTITUDE_SIGNAL.try_take().is_none(), "LATEST_ALTITUDE_SIGNAL should be empty after takes");
    }

    #[test_log::test(rstest::rstest)]
    #[async_std::test]
    #[serial_test::serial]
    #[case(random_altimeter_data())]
    #[case(random_gps_data())]
    #[case(random_imu_data())]
    #[case(FlightState::default())]
    #[timeout(ms(100))]
    async fn record_to_storage_channel_max_capacity(
        #[case] record: impl Into<Record>, 
    ) {
        let record = record.into();

        // First, fill the channel to capacity
        for _ in 0..(2*TasksConfig::RECORD_TO_STORAGE_CHANNEL_DEPTH) {
            broadcast_record(record.clone());
        }

        // Now, receive all records and ensure they are correct
        for _ in 0..TasksConfig::RECORD_TO_STORAGE_CHANNEL_DEPTH {
            let received_record = RECORD_TO_STORAGE_CHANNEL.receiver().receive().await;
            assert_eq!(received_record, record);
        }
        assert!(RECORD_TO_STORAGE_CHANNEL.receiver().try_receive().is_err(), "Channel should be empty after receiving all records");
    }    
}
