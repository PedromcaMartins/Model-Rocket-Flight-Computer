use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// A rolling-window buffer generic over the element type.
///
/// Evicts samples older than `window` on each `push()` — no max length cap.
pub struct RollingHistory<T> {
    samples: VecDeque<(Instant, T)>,
    window: Duration,
}

impl<T: Clone> RollingHistory<T> {
    pub fn new(window: Duration) -> Self {
        Self {
            samples: VecDeque::new(),
            window,
        }
    }

    pub fn push(&mut self, time: Instant, value: T) {
        let cutoff = time - self.window;
        while let Some(front) = self.samples.front() {
            if front.0 < cutoff {
                self.samples.pop_front();
            } else {
                break;
            }
        }

        self.samples.push_back((time, value));
    }

    pub fn latest(&self) -> Option<&T> {
        self.samples.back().map(|(_, v)| v)
    }

    pub fn snapshot(&self) -> Vec<(Instant, T)> {
        self.samples.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

impl<T: Clone + PartialOrd> RollingHistory<T> {
    pub fn min(&self) -> Option<&T> {
        self.samples.iter().map(|(_, v)| v).min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn max(&self) -> Option<&T> {
        self.samples.iter().map(|(_, v)| v).max_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn push_and_snapshot() {
        let mut h: RollingHistory<f32> = RollingHistory::new(Duration::from_secs(30));
        let now = Instant::now();
        h.push(now, 1.0);
        h.push(now + Duration::from_secs(1), 2.0);
        h.push(now + Duration::from_secs(2), 3.0);
        assert_eq!(h.len(), 3);
        assert_eq!(h.snapshot().len(), 3);
    }

    #[test]
    fn evicts_old_samples() {
        let mut h: RollingHistory<f32> = RollingHistory::new(Duration::from_secs(10));
        let now = Instant::now();
        h.push(now, 1.0);
        h.push(now + Duration::from_secs(5), 2.0);
        h.push(now + Duration::from_secs(15), 3.0);
        // t=5 is exactly `window` before t=15 — boundary is kept (strict <).
        assert_eq!(h.len(), 2);
        assert_eq!(*h.latest().unwrap(), 3.0);
    }

    #[test]
    fn latest_returns_most_recent() {
        let mut h: RollingHistory<i32> = RollingHistory::new(Duration::from_secs(30));
        let now = Instant::now();
        h.push(now, 10);
        h.push(now + Duration::from_secs(1), 20);
        h.push(now + Duration::from_secs(2), 30);
        assert_eq!(*h.latest().unwrap(), 30);
    }

    #[test]
    fn window_eviction_and_growth() {
        let mut h: RollingHistory<u32> = RollingHistory::new(Duration::from_secs(60));
        let now = Instant::now();
        h.push(now, 1);
        h.push(now + Duration::from_secs(1), 2);
        h.push(now + Duration::from_secs(2), 3);
        h.push(now + Duration::from_secs(3), 4);
        assert_eq!(h.len(), 4);
        assert_eq!(*h.latest().unwrap(), 4);
    }
}
