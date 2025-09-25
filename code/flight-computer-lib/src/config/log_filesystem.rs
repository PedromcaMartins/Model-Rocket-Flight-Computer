use embassy_time::Duration;
use telemetry_messages::LogDataType;

#[derive(Copy, Clone)]
pub struct LogFileSystemConfig {
    pub flush_files_ticker_period: Duration,
}

impl Default for LogFileSystemConfig {
    fn default() -> Self {
        Self {
            flush_files_ticker_period: Duration::from_millis(500),
        }
    }
}

impl LogFileSystemConfig {
    pub const FNV_INDEX_MAP_SIZE: usize = {
        const fn next_power_of_two(n: usize) -> usize {
            if n <= 1 {
                2
            } else {
                1 << (usize::BITS - (n - 1).leading_zeros())
            }
        }
        const SIZE: usize = next_power_of_two(LogDataType::LENGTH);

        // Verify it's a power of 2
        const _: () = assert!(SIZE > 0 && SIZE.is_power_of_two());
        // Verify it's greater than LogDataType::LENGTH
        const _: () = assert!(SIZE > LogDataType::LENGTH);

        SIZE
    };

    pub const WRITE_BUFFER_SIZE: usize = 576;

    pub const MAX_FILENAME_LENGTH: usize = 8;
    pub const MAX_UID_LENGTH: usize = Self::MAX_FILENAME_LENGTH - LogDataType::MAX_BASE_FILENAME_LENGTH;
}
