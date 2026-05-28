// Shared constants extracted from cross-crate duplication.
// Consumers use `utils::constants::*`; do not re-define these locally.

/// IPC socket name for the flight-computer ↔ simulator link.
pub const SIM_SOCKET_NAME: &str = "fc-sim.sock";

/// IPC socket name for the flight-computer ↔ ground-station link.
pub const GS_SOCKET_NAME: &str = "fc-gs.sock";

/// Default stdout log level when RUST_LOG is unset.
pub const STDOUT_LOG_LEVEL: tracing::level_filters::LevelFilter =
    tracing::level_filters::LevelFilter::INFO;

/// Timestamp format for log directories and session records.
pub const TIMESTAMP_FORMAT: &str = "%Y_%m_%d_%H_%M_%S";

// Log buffer capacity.
pub(crate) const DEFAULT_BUFFER_CAPACITY: usize = 10_000;
