use std::time::Duration;

use utils::constants as c;

pub struct Config;

impl Config {
    // -- TUI --
    pub const TUI_FPS: u16 = 60;
    pub const HISTORY_WINDOW: Duration = Duration::from_secs(30);
    pub const LOG_BUFFER_CAPACITY: usize = 1000;
    pub const LOG_VISIBLE_LINES: usize = 20;

    // -- Reconnection --
    pub const RECONNECT_INTERVAL: Duration = Duration::from_secs(1);

    // -- URL helpers --
    const WS_SCHEME: &str = "ws";
    const HTTP_SCHEME: &str = "http";

    fn url(scheme: &str, path: &str) -> String {
        format!("{scheme}://{0}:{1}{2}{path}", c::GS_HOST, c::GS_PORT, c::API_PATH)
    }

    pub fn ws_url() -> String { Self::url(Self::WS_SCHEME, c::WS_PATH) }
    pub fn arm_url() -> String { Self::url(Self::HTTP_SCHEME, c::ARM_PATH) }
    pub fn ignite_url() -> String { Self::url(Self::HTTP_SCHEME, c::IGNITE_PATH) }
    pub fn ping_url() -> String { Self::url(Self::HTTP_SCHEME, c::PING_PATH) }
}
