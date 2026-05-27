//! Compile-time constants for the ground-station backend.
//!
//! All values are `pub const` on a unit struct per AGENTS.md §6 layout.
//! Config is a namespace, never instantiated.

use std::time::Duration;

/// Infrastructure configuration (sockets, REST, storage paths).
pub struct Config;

impl Config {
    // -- FC connection --
    /// Namespaced local-socket path for the FC ↔ GS link.
    pub const FC_SOCKET_PATH: &str = "fc-gs.sock";
    /// Depth of the postcard-rpc outgoing message queue.
    pub const CLIENT_QUEUE_DEPTH: usize = 1024;
    /// Timeout for endpoint calls (e.g. ping).
    pub const ENDPOINT_TIMEOUT: Duration = Duration::from_secs(2);

    // -- REST server --
    pub const REST_HOST: &str = "127.0.0.1";
    pub const REST_PORT: u16 = 8000;
    pub const CTRLC: bool = true;
    pub const GRACE: u64 = 5;
    pub const MERCILESS: bool = false;
    pub const API_PATH: &str = "/api";

    // -- Record storage --
    /// Directory under CWD where session NDJSON files are stored.
    pub const RECORDS_ROOT_DIR: &str = "logs/gs_records";
    /// Timestamp format used in session directory names.
    pub const RECORDS_TIMESTAMP_FORMAT: &str = "%Y_%m_%d_%H_%M_%S";

    // -- Logging --
    /// Directory under CWD where per-level JSON logs are stored.
    pub const LOG_ROOT_DIR: &str = "logs/gs_backend";
    /// Timestamp format used in log session directory names.
    pub const LOG_TIMESTAMP_FORMAT: &str = "%Y_%m_%d_%H_%M_%S";
    /// Default RUST_LOG level for stdout when the env-var is unset.
    pub const STDOUT_LOG_LEVEL: tracing::level_filters::LevelFilter =
        tracing::level_filters::LevelFilter::INFO;
}
