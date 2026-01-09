use derive_more::From;

use crate::{Serialize, Deserialize, Schema};

#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalTickHz{ hz: u64, nano_hz: u32 }

#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq, Eq, From)]
pub struct Timestamp{ ticks: u64 }

#[cfg(feature = "timestamp-into-duration")]
mod into_duration_impls {
    use core::time::Duration;

    use super::{GlobalTickHz, Timestamp};

    impl Timestamp {
        /// Converts the timestamp into a `Duration` since startup, given the global tick frequency.
        /// 
        /// # Errors
        /// Returns an error if the conversion to `u32` for nanoseconds fails.
        #[must_use]
        #[allow(clippy::cast_possible_truncation)]
        pub const fn duration_since_startup(&self, global_tick: GlobalTickHz) -> Duration {
            Duration::new(
                self.ticks / global_tick.hz,
                (self.ticks % global_tick.hz) as u32 * global_tick.nano_hz,
            )
        }
    }
}

#[cfg(feature = "embassy-time")]
mod embassy_impls {
    use embassy_time::Instant;

    use super::{GlobalTickHz, Timestamp};

    impl GlobalTickHz {
        /// Sets the global tick frequency in Hz.
        /// 
        /// # Panics
        /// Panics if `hz` is zero.
        #[must_use]
        #[allow(clippy::cast_possible_truncation)]
        pub fn set_global_tick_hz(hz: u64) -> Self {
            assert_ne!(hz, 0, "Tick Hz must be non-zero");
            Self {
                hz,
                nano_hz: (1_000_000_000 / hz) as u32,
            }
        }
    }

    impl Timestamp {
        #[inline]
        #[must_use]
        pub fn now() -> Self {
            Self {
                ticks: Instant::now().as_ticks(),
            }
        }
    }
}
