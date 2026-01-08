use crate::{Deserialize, Serialize, Schema};

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    FileSystem(FileSystemError),
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Debug, PartialEq, Eq)]
pub enum FileSystemError {
    FilenameTooLong,
    GetUniqueIdFailed,
    UniqueIdUnavailable,
    FileCreationFailed,
    FailedToSerializeRecord,
    WriteBufferTooSmall,
    FailedToWriteRecord,
    FailedToFlushFile,
}
