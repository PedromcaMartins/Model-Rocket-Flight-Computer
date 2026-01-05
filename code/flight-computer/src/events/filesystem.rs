use proto::LogDataType;

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileSystemSuccess {
    UniqueFilesCreated,
    MessageAppended(LogDataType),
    FilesFlushed,
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileSystemError {
    FilenameTooLong,
    GetUniqueIdFailed,
    UniqueIdUnavailable,
    FileCreationFailed(LogDataType),
    FileHandleAlreadyExists(LogDataType),
    StoreFileHandleFailed(LogDataType),
    FileHandleNotFound(LogDataType),
    FailedToSerializeMessage(LogDataType),
    FailedToWriteMessage(LogDataType),
    FailedToFlushFile(LogDataType),
}

pub type FileSystemResult = Result<FileSystemSuccess, FileSystemError>;
