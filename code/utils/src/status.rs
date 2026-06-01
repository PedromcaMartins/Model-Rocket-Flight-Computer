#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Status {
    pub connected: bool,
    pub session_start: chrono::DateTime<chrono::Utc>,
    pub record_count: u64,
    pub latency: Option<std::time::Duration>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PingSuccess {
    pub latency: std::time::Duration,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct CommandSuccess {
    pub status: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct CommandError {
    pub error: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum WsMessage {
    Record(proto::record::Record),
    Log(String),
    Status(Status),
}
