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
        let end = now();
        log_record(
            self.function, 
            Some(self.stage), 
            self.start, 
            end
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
    use super::*;

    #[test_log::test]
    fn test_sync() {
        unimplemented!()
    }

    #[test_log::test(async_std::test)]
    async fn test_async() {
        unimplemented!()
    }
}
