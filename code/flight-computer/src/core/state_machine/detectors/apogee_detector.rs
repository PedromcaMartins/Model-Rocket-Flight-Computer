use embassy_time::{Instant, Ticker};
use heapless::HistoryBuf;
use proto::sensor_data::{Altitude, Time, Velocity};
use proto::uom::si::time::microsecond;

use crate::config::ApogeeDetectorConfig;
use crate::sync::LATEST_ALTITUDE_SIGNAL;

pub struct ApogeeDetector {
    launchpad_altitude: Altitude,

    altitude_buffer: HistoryBuf<Altitude, { ApogeeDetectorConfig::ALTITUDE_BUFFER_SIZE }>,
    velocity_buffer: HistoryBuf<Velocity, { ApogeeDetectorConfig::VELOCITY_BUFFER_SIZE }>,
    prev_data: (Altitude, Instant),
}

impl ApogeeDetector {
    pub async fn new(
        launchpad_altitude: Altitude,
    ) -> Self {
        let altitude = LATEST_ALTITUDE_SIGNAL.wait().await;
        let altitude_above_launchpad = altitude - launchpad_altitude;

        Self {
            launchpad_altitude,

            altitude_buffer: HistoryBuf::new(),
            velocity_buffer: HistoryBuf::new(),
            prev_data: (altitude_above_launchpad, Instant::now()),
        }
    }

    fn are_buffers_full(&self) -> bool {
        self.altitude_buffer.is_full() &&
        self.velocity_buffer.is_full()
    }

    async fn get_altitude_above_launchpad(&self) -> Altitude {
        let raw_altitude = LATEST_ALTITUDE_SIGNAL.wait().await;
        raw_altitude - self.launchpad_altitude
    }

    fn get_latest_altitude_above_launchpad(&self) -> Altitude {
        self.prev_data.0
    }

    async fn wait_new_data_and_update_buffers(&mut self) {
        // delta_h
        let prev_altitude = self.get_latest_altitude_above_launchpad();
        let altitude = self.get_altitude_above_launchpad().await;
        let delta_h = altitude - prev_altitude;

        // delta_t
        let now = Instant::now();
        let delta_t = now - self.prev_data.1;
        let delta_t = Time::new::<microsecond>(delta_t.as_micros() as f32);

        // calculate velocity
        let velocity = delta_h / delta_t;

        // update buffers
        self.altitude_buffer.write(altitude);
        self.velocity_buffer.write(velocity);
        self.prev_data = (altitude, now);
    }

    pub async fn await_apogee(&mut self) -> Altitude {
        let mut ticker = Ticker::every(ApogeeDetectorConfig::DETECTOR_TICK_INTERVAL);

        loop {
            ticker.next().await;
            self.wait_new_data_and_update_buffers().await;

            // Check if buffers are full before evaluating conditions
            if self.are_buffers_full() {
                let descent_vel_check = self.velocity_buffer.iter().all(
                    |&v| v <= ApogeeDetectorConfig::max_descent_velocity()
                );

                let minimum_altitude_check = self.altitude_buffer.iter().all(
                    |&h| h >= ApogeeDetectorConfig::min_apogee_altitude_above_launchpad()
                );

                if descent_vel_check && minimum_altitude_check {
                    return self.get_latest_altitude_above_launchpad();
                }
            }
        }
    }
}
