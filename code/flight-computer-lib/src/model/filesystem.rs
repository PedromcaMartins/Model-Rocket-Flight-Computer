#[derive(enum_map::Enum)]
pub enum LogDataType {
    Altimeter,
    Gps,
    Imu,
}

pub trait FileSystem {
    type Error: core::fmt::Debug;

    fn append_message<T: telemetry_messages::Serialize>(
        &mut self,
        log_data_type: LogDataType,
        log_data: &T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error>;

    fn flush_all_files(&mut self) -> Result<(), Self::Error>;
    fn reopen_all_files(&mut self) -> Result<(), Self::Error>;
}
