use telemetry_messages::LogDataType;

#[derive(Copy, Clone, Default)]
pub struct LogFileSystemConfig;

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
}
