use crate::interfaces::Sensor;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

mod altimeter;
pub use altimeter::SimAltimeter;
mod gps;
pub use gps::SimGps;
mod imu;
pub use imu::SimImu;

pub trait SimSensor : Sensor + Default {
    fn signal() -> &'static Signal<CriticalSectionRawMutex, Self::Data>;

    fn update_data(data: Self::Data) {
        Self::signal().signal(data);
    }
}

#[cfg(test)]
mod tests {    
    use futures::FutureExt;
    use rstest::fixture;
    use crate::test_utils::{ms, sensor_data::{random_altimeter_data, random_gps_data, random_imu_data}};

    use super::*;

    #[fixture]
    fn sim_sensor<T: SimSensor>() -> T {
        T::signal().reset();
        T::default()
    }

    async fn update_and_parse<T: SimSensor>(data: T::Data, sim_sensor: &mut T) {
        T::update_data(data.clone());

        let parsed_data = sim_sensor
            .parse_new_data()
            .await
            .expect("Failed to parse new data");

        assert_eq!(parsed_data, data, "Parsed data ({parsed_data:?}) does not match expected data ({data:?})");
    }

    async fn timeout_when_no_data<T: SimSensor>(mut sim_sensor: T) {
        let _ = sim_sensor.parse_new_data().await;
        panic!("parse_new_data should have blocked waiting for data");
    }

    #[test_log::test(rstest::rstest)]
    #[async_std::test]
    #[serial_test::serial]
    #[case(1)]
    #[case(10)]
    #[case(1_000)]
    #[timeout(ms(100))]
    async fn test_sim_sensor_update_and_parse(
        #[case] updates: usize,
        #[from(sim_sensor)] mut sim_altimeter: SimAltimeter,
        #[from(sim_sensor)] mut sim_gps: SimGps,
        #[from(sim_sensor)] mut sim_imu: SimImu,
    ) {
        for _ in 0..updates {
            update_and_parse(random_altimeter_data(), &mut sim_altimeter).await;
            update_and_parse(random_gps_data(), &mut sim_gps).await;
            update_and_parse(random_imu_data(), &mut sim_imu).await;
        }
    }

    #[test_log::test(rstest::rstest)]
    #[async_std::test]
    #[serial_test::serial]
    #[timeout(ms(100))]
    #[should_panic(expected = "Timeout 100ms expired")]
    async fn test_sim_sensor_signal_blocks_when_no_data(
        #[from(sim_sensor)] sim_altimeter: SimAltimeter,
        #[from(sim_sensor)] sim_gps: SimGps,
        #[from(sim_sensor)] sim_imu: SimImu,
    ) {
        futures::select! {
            () = timeout_when_no_data(sim_altimeter).fuse() => (),
            () = timeout_when_no_data(sim_gps).fuse() => (),
            () = timeout_when_no_data(sim_imu).fuse() => (),
        }
    }
}
