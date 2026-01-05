use core::cmp::Ordering;

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::{Instant, Ticker};
use heapless::HistoryBuf;
use proto::{Altitude, Time, Velocity};
use proto::uom::si::time::microsecond;

use crate::config::TouchdownDetectorConfig;
use crate::core::trace::TraceAsync;

pub struct TouchdownDetector<M>
where
    M: RawMutex + 'static,
{
    altitude_signal: &'static Signal<M, Altitude>,
    config: TouchdownDetectorConfig,

    altitude_buffer: HistoryBuf<Altitude, { TouchdownDetectorConfig::ALTITUDE_BUFFER_SIZE }>,
    velocity_buffer: HistoryBuf<Velocity, { TouchdownDetectorConfig::VELOCITY_BUFFER_SIZE }>,
    prev_data: (Altitude, Instant),
}

impl<M> TouchdownDetector<M>
where
    M: RawMutex + 'static,
{
    pub async fn new(
        altitude_signal: &'static Signal<M, Altitude>,
        config: TouchdownDetectorConfig,
    ) -> Self {
        let altitude = altitude_signal.wait().await;

        Self {
            altitude_signal,
            config,

            altitude_buffer: HistoryBuf::new(),
            velocity_buffer: HistoryBuf::new(),
            prev_data: (altitude, Instant::now()),
        }
    }

    fn are_buffers_full(&self) -> bool {
        self.altitude_buffer.is_full() &&
        self.velocity_buffer.is_full()
    }

    fn get_latest_altitude(&self) -> Altitude {
        self.prev_data.0
    }

    async fn wait_new_data_and_update_buffers(&mut self) {
        // delta_h
        let prev_altitude = self.get_latest_altitude();
        let altitude = self.altitude_signal.wait().await;
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

    pub async fn await_touchdown(&mut self) -> Altitude {
        let mut ticker = Ticker::every(self.config.detector_tick_period);

        loop {
            let mut trace = TraceAsync::start("imu_task_loop");

            trace.before_await();
            ticker.next().await;
            self.wait_new_data_and_update_buffers().await;
            trace.after_await();

            // Check if buffers are full before evaluating conditions
            if self.are_buffers_full() {
                let min_altitude = self.altitude_buffer.iter()
                    .min_by(|&x, &y| x.partial_cmp(y).unwrap_or(Ordering::Equal)).expect("Buffer is full");
                let max_altitude = self.altitude_buffer.iter()
                    .max_by(|&x, &y| x.partial_cmp(y).unwrap_or(Ordering::Equal)).expect("Buffer is full");

                let touchdown_stability_check = (*max_altitude - *min_altitude).abs() <= self.config.touchdown_stability_threshold;

                let touchdown_velocity_check = self.velocity_buffer.iter()
                    .all(|&vel| vel.abs() <= self.config.touchdown_velocity_threshold);

                if touchdown_stability_check && touchdown_velocity_check {
                    return self.get_latest_altitude();
                }
            }
        }
    }
}
