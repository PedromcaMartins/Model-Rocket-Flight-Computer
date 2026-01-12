#![allow(dead_code)]

type Timestamp = u64;

fn log_record(fn_name: &'static str, stage: Option<u64>, start: Timestamp, end: Timestamp) {
    defmt_or_log::trace!("trace: {} {:?} {} {}", fn_name, stage, start, end);
}

fn now() -> Timestamp {
    embassy_time::Instant::now().as_ticks()
}

pub struct TraceSync {
    function: &'static str,
    start: Timestamp,
}

impl TraceSync {
    pub fn start(function: &'static str) -> Self {
        Self {
            function,
            start: now(),
        }
    }
}

impl Drop for TraceSync {
    fn drop(&mut self) {
        let end = now();
        log_record(self.function, None, self.start, end);
    }
}

pub struct TraceAsync {
    function: &'static str,
    start: Timestamp,
    // even numbers represent syncronous execution time, odd numbers represent await time
    stage: u64,
}

impl TraceAsync {
    /// Starts a new trace
    pub fn start(function: &'static str) -> Self {
        Self {
            function,
            start: now(),
            stage: 0,
        }
    }

    /// Pause before an await
    pub fn before_await(&mut self) {
        self.restart();
    }

    /// Resume after an await
    pub fn after_await(&mut self) {
        self.restart();
    }

    fn log_record(&self) {
        log_record(
            self.function, 
            Some(self.stage), 
            self.start, 
            now(),
        );
    }

    fn restart(&mut self) {
        self.log_record();
        self.start = now();
        self.stage += 1;
    }
}

impl Drop for TraceAsync {
    fn drop(&mut self) {
        self.log_record();
    }
}

#[cfg(test)]
mod tests {
    use embassy_time::{Duration, MockDriver};
    use logtest::Logger;

    use crate::test_utils::{mock_driver, mock_logger};

    use super::*;

    fn assert_log_record(record: &logtest::Record, fn_name: &str, stage: Option<u64>, start: Timestamp, end: Timestamp) {
        let msg = record.args();
        assert_eq!(record.level(), log::Level::Trace);
        assert!(msg.contains(fn_name));
        assert!(msg.contains(&format!("{stage:?}")));
        assert!(msg.contains(&start.to_string()));
        assert!(msg.contains(&end.to_string()));
    }

    #[rstest::rstest]
    #[serial_test::serial]
    #[case("my_fn", Some(3), 10, 20)]
    #[case("other_fn", None, 0, 42)]
    #[case("compute", Some(0), 5, 5)]
    fn log_record_emits_trace(
        #[case] name: &'static str,
        #[case] stage: Option<u64>,
        #[case] start: u64,
        #[case] end: u64,
        mut mock_logger: Logger,
    ) {
        log_record(name, stage, start, end);

        let record = mock_logger.pop().expect("expected log record");
        assert_log_record(&record, name, stage, start, end);
    }

    #[rstest::rstest]
    #[serial_test::serial]
    #[case("compute_heavy", 10, 50)]
    #[case("fast", 0, 1)]
    #[case::panic("time_travel", 42, 0)]
    fn trace_sync(
        #[case] fn_name: &'static str,
        #[case] start_at: u64,
        #[case] duration: u64,
        mock_driver: &MockDriver,
        mut mock_logger: Logger,
    ) {
        // Arrange
        mock_driver.advance(Duration::from_ticks(start_at));
        let start = now();

        {
            let _trace = TraceSync::start(fn_name);
            mock_driver.advance(Duration::from_ticks(duration));
        } // <- drop happens here

        let end = now();

        // Assertions
        let record = mock_logger.pop().expect("expected log record");
        assert_log_record(&record, fn_name, None, start, end);
        assert!(mock_logger.pop().is_none());
    }

    #[rstest::rstest]
    #[async_std::test]
    #[serial_test::serial]
    #[case("single", &[10, 20, 30])] // before, await, drop
    #[case("double", &[5, 10, 15, 20, 25])] // before, await, before, await, drop
    #[case("zero", &[0, 0, 0])] // zero-duration everywhere
    #[case("sync", &[50])] // only drop
    async fn trace_async(
        #[case] fn_name: &'static str,
        #[case] advances: &[u64],
        mock_driver: &MockDriver,
        mut mock_logger: Logger,
    ) {
        // Arrange
        {
            let mut trace = TraceAsync::start(fn_name);

            for (i, &advance) in advances.iter().enumerate() {
                println!("Stage {i}: advancing {advance} ticks");
                mock_driver.advance(Duration::from_ticks(advance));

                // Last advance before drop
                if i == advances.len() - 1 {
                    break;
                }
                if i % 2 == 0 {
                    trace.before_await();
                } else {
                    trace.after_await();
                }
            }
        } // <- drop happens here

        for idx in 0..advances.len() {
            let r = mock_logger.pop().unwrap_or_else(|| panic!("expected log record {idx}"));
            assert_log_record(
                &r,
                fn_name,
                Some(idx as u64),
                advances[..idx].iter().sum(),
                advances[..=idx].iter().sum(),
            );
        }
        assert!(mock_logger.pop().is_none(), "expected exactly {} log records", advances.len());
    }
}
