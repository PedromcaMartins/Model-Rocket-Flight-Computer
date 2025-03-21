use std::path::{Path, PathBuf};

use defmt_decoder::{Frame, Location, Locations};

#[derive(Debug)]
pub struct LogMessage {
    pub timestamp: String,
    pub level: Option<defmt_parser::Level>,
    pub message: String,
    pub location: Option<Location>, 
}

impl LogMessage {
    pub fn new(frame: &Frame, locs: &Option<Locations>) -> Self {
        let location = locs.as_ref()
        .and_then(|locs| locs.get(&frame.index()))
        .map(|loc| {
            let path = to_relative_from_src(&loc.file).unwrap_or_else(|| loc.file.clone());
            Location {
                file: path,
                line: loc.line,
                module: loc.module.clone(),
            }
        });

        Self {
            timestamp: frame
                .display_timestamp()
                .map(|ts| ts.to_string())
                .unwrap_or_default(),
            level: frame.level(),
            message: frame.display_message().to_string(),
            location,
        }
    }
}

fn to_relative_from_src(absolute_path: &Path) -> Option<PathBuf> {
    // Find "src" in the path components
    for (i, component) in absolute_path.components().enumerate() {
        if component.as_os_str() == "src" {
            // Create a relative path after "src/"
            return Some(absolute_path.iter().skip(i + 1).collect());
        }
    }
    
    None // "src" not found in the path
}
