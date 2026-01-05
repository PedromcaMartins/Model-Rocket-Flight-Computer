use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::{Instant, Ticker};
use heapless::HistoryBuf;
use proto::{Altitude, Time, Velocity};
use proto::uom::si::time::microsecond;

use crate::config::ApogeeDetectorConfig;
use crate::core::trace::TraceAsync;

pub struct ApogeeDetector<M>
where
    M: RawMutex + 'static,
{
    altitude_signal: &'static Signal<M, Altitude>,
    launchpad_altitude: Altitude,
    config: ApogeeDetectorConfig,

    altitude_buffer: HistoryBuf<Altitude, { ApogeeDetectorConfig::ALTITUDE_BUFFER_SIZE }>,
    velocity_buffer: HistoryBuf<Velocity, { ApogeeDetectorConfig::VELOCITY_BUFFER_SIZE }>,
    prev_data: (Altitude, Instant),
}

impl<M> ApogeeDetector<M>
where
    M: RawMutex + 'static,
{
    pub async fn new(
        altitude_signal: &'static Signal<M, Altitude>,
        launchpad_altitude: Altitude,
        config: ApogeeDetectorConfig,
    ) -> Self {
        let altitude = altitude_signal.wait().await;
        let altitude_above_launchpad = altitude - launchpad_altitude;

        Self {
            altitude_signal,
            launchpad_altitude,
            config,

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
        let raw_altitude = self.altitude_signal.wait().await;
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
        let mut ticker = Ticker::every(self.config.detector_tick_period);

        loop {
            let mut trace = TraceAsync::start("imu_task_loop");

            trace.before_await();
            ticker.next().await;
            self.wait_new_data_and_update_buffers().await;
            trace.after_await();

            // Check if buffers are full before evaluating conditions
            if self.are_buffers_full() {
                let descent_vel_check = self.velocity_buffer.iter().all(
                    |&v| v <= self.config.max_descent_velocity
                );

                let minimum_altitude_check = self.altitude_buffer.iter().all(
                    |&h| h >= self.config.min_apogee_altitude_above_launchpad
                );

                if descent_vel_check && minimum_altitude_check {
                    return self.get_latest_altitude_above_launchpad();
                }
            }
        }
    }
}
