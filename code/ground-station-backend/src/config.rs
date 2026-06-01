//! Compile-time constants for the ground-station backend.
//!
//! All values are `pub const` on a unit struct per AGENTS.md §6 layout.
//! Config is a namespace, never instantiated.

use std::path::PathBuf;
use std::time::Duration;

/// Infrastructure configuration (sockets, REST, storage paths).
pub struct Config;

impl Config {
    // -- FC connection --
    /// Depth of the postcard-rpc outgoing message queue.
    pub const CLIENT_QUEUE_DEPTH: usize = 1024;
    /// Timeout for endpoint calls (e.g. ping).
    pub const ENDPOINT_TIMEOUT: Duration = Duration::from_secs(2);
    /// Interval between FC ping measurements.
    pub const PING_POLL: Duration = Duration::from_secs(1);
    /// Echo payload for FC ping endpoint.
    pub const PING_PAYLOAD: u32 = 0xdeadbeef;
    /// Delay between reconnection attempts after a FC disconnect.
    pub const RECONNECT_INTERVAL: Duration = Duration::from_secs(1);

    // -- REST server --
    pub const CTRLC: bool = true;
    pub const GRACE: u64 = 5;
    pub const MERCILESS: bool = false;

    // -- Record storage --
    /// Absolute path to the session NDJSON storage directory,
    /// anchored under `code/logs/gs_records` at compile time.
    pub fn records_root_dir() -> PathBuf {
        utils::workspace::workspace_root().join("logs").join("gs_records")
    }
    /// Timestamp format used in session directory names.
    pub const RECORDS_TIMESTAMP_FORMAT: &str = utils::constants::TIMESTAMP_FORMAT;

    // -- Logging --
    /// Default RUST_LOG level for stdout when the env-var is unset.
    pub const STDOUT_LOG_LEVEL: tracing::level_filters::LevelFilter =
        utils::constants::STDOUT_LOG_LEVEL;
}
