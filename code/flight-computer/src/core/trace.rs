#![allow(dead_code)]

type Timestamp = u64;

fn log_record(fn_name: &'static str, stage: Option<u64>, start: Timestamp, end: Timestamp) {
    crate::log::trace!("trace: {} {:?} {} {}", fn_name, stage, start, end);
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
    use logtest::Logger;

    use crate::test_utils::mock_logger;

    use super::*;

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
        let msg = record.args();
        assert_eq!(record.level(), log::Level::Trace);
        assert!(msg.contains(name));
        assert!(msg.contains(&format!("{stage:?}")));
        assert!(msg.contains(&start.to_string()));
        assert!(msg.contains(&end.to_string()));
    }

    #[rstest::rstest]
    #[serial_test::serial]
    #[case("compute_heavy")]
    #[case("fast")]
    #[case::panic("time_travel")]
    fn trace_sync(
        #[case] fn_name: &'static str,
        mut mock_logger: Logger,
    ) {
        {
            let _trace = TraceSync::start(fn_name);
        } // <- drop happens here

        let record = mock_logger.pop().expect("expected log record");
        let msg = record.args();
        assert_eq!(record.level(), log::Level::Trace);
        assert!(msg.contains(fn_name));
        assert!(msg.contains("None"));
        assert!(mock_logger.pop().is_none());
    }

    #[rstest::rstest]
    #[async_std::test]
    #[serial_test::serial]
    #[case("three_stages", 3)]
    #[case("five_stages", 5)]
    #[case("only_drop", 1)]
    async fn trace_async(
        #[case] fn_name: &'static str,
        #[case] expected_records: usize,
        mut mock_logger: Logger,
    ) {
        {
            let mut trace = TraceAsync::start(fn_name);
            for i in 0..expected_records.saturating_sub(1) {
                if i % 2 == 0 {
                    trace.before_await();
                } else {
                    trace.after_await();
                }
            }
        } // <- drop happens here

        for idx in 0..expected_records {
            let r = mock_logger.pop()
                .unwrap_or_else(|| panic!("expected log record {idx}"));
            let msg = r.args();
            assert_eq!(r.level(), log::Level::Trace);
            assert!(msg.contains(fn_name));
            assert!(
                msg.contains(&format!("Some({idx})")),
                "stage Some({idx}) not found in '{msg}'"
            );
        }
        assert!(
            mock_logger.pop().is_none(),
            "expected exactly {expected_records} log records"
        );
    }
}
