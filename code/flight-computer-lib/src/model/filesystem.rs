#[defmt_or_log_macros::maybe_derive_format]
#[derive(enum_map::Enum, Debug, Clone, Copy)]
pub enum LogDataType {
    Altimeter,
    Gps,
    Imu,
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(enum_map::Enum, Debug, Clone, Copy)]
pub enum FileSystemEvent {
    MessageWritten,
    FailedToSerializeMessage,
    FailedToWriteMessage,
    FileFlushed,
    FailedToFlushFile,
    FileReopened,
    FailedToReopenFile,
    Other,
}

pub trait FileSystem {
    fn append_message<T: telemetry_messages::Serialize>(
        &mut self,
        log_data_type: LogDataType,
        log_data: &T,
        buffer: &mut [u8],
    ) -> FileSystemEvent;

    fn flush_file(&mut self, log_data_type: LogDataType) -> FileSystemEvent;

    fn reopen_file(&mut self, log_data_type: LogDataType) -> FileSystemEvent;
}
