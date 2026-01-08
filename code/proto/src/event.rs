use crate::{Deserialize, Serialize, Schema};

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    FileSystem(FileSystemEvent),
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq, Eq)]
pub enum FileSystemEvent {
    UniqueFileCreated,
    RecordAppended,
    FileFlushed,
}
