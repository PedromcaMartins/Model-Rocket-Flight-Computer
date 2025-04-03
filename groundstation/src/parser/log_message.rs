use std::{convert::Infallible, fmt, path::{Path, PathBuf}, str::FromStr};
use telemetry::{AltimeterMessage, GpsMessage, ImuMessage};
use time::OffsetDateTime;

pub enum MessageType {
    AltimeterMessage(AltimeterMessage),
    GpsMessage(GpsMessage),
    ImuMessage(ImuMessage),
    String(String),
}

impl FromStr for MessageType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(msg) = s.parse::<AltimeterMessage>() {
            Ok(Self::AltimeterMessage(msg))
        } else if let Ok(msg) = s.parse::<GpsMessage>() {
            Ok(Self::GpsMessage(msg))
        } else if let Ok(msg) = s.parse::<ImuMessage>() {
            Ok(Self::ImuMessage(msg))
        } else {
            Ok(Self::String(s.to_string()))
        }
    }
}

impl std::fmt::Debug for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AltimeterMessage(msg) => write!(f, "{:?}", msg),
            Self::GpsMessage(msg) => write!(f, "{:?}", msg),
            Self::ImuMessage(msg) => write!(f, "{:?}", msg),
            Self::String(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug)]
pub struct LogMessage {
    pub timestamp: String,
    /// Unix timestamp in nanoseconds
    pub host_timestamp: i64,
    pub level: Option<defmt_parser::Level>,
    pub message: MessageType,
    pub location: Option<Location>,
}

#[derive(Clone, Debug)]
pub struct Location {
    pub file_complete_path: PathBuf,
    pub file: String,
    pub line: u32,
    pub module_path: Option<ModulePath>,
}

#[derive(Clone, Debug)]
pub struct ModulePath {
    pub crate_name: String,
    pub modules: Vec<String>,
    pub function: String,
}

impl LogMessage {
    pub fn new(frame: &defmt_decoder::Frame, locs: &Option<defmt_decoder::Locations>, current_dir: &Path) -> Self {
        let location = locs.as_ref()
        .and_then(|locs| locs.get(&frame.index()))
        .map(|loc| {
            // try to get the relative path, else the full one
            let path = loc.file.strip_prefix(current_dir).unwrap_or(&loc.file);
            Location {
                file_complete_path: loc.file.clone(),
                file: path.display().to_string(),
                line: loc.line as u32,
                module_path: create_module_path(&loc.module),
            }
        });

        let host_timestamp = OffsetDateTime::now_utc()
        .unix_timestamp_nanos()
        .min(i64::MAX as i128) as i64;

        let timestamp = frame
        .display_timestamp()
        .map(|ts| ts.to_string())
        .unwrap_or_default();

        Self {
            host_timestamp,
            timestamp,
            level: frame.level(),
            message: frame.display_message().to_string().parse::<MessageType>().unwrap(),
            location,
        }
    }
}

fn create_module_path(module_path: &str) -> Option<ModulePath> {
    let mut path = module_path.split("::").collect::<Vec<_>>();

    // there need to be at least two elements, the crate and the function
    if path.len() < 2 {
        return None;
    };

    // the last element is the function
    let function = path.pop()?.to_string();
    // the first element is the crate_name
    let crate_name = path.remove(0).to_string();

    Some(ModulePath {
        crate_name,
        modules: path.into_iter().map(|a| a.to_string()).collect(),
        function,
    })
}
