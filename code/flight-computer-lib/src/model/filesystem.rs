use enum_map::Enum;

pub mod log_filesystem;

#[defmt_or_log_macros::maybe_derive_format]
#[derive(enum_map::Enum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogDataType {
    Altimeter,
    Gps,
    Imu,
}

impl LogDataType {
    pub const VALUES: [Self; Self::LENGTH] = [
        Self::Altimeter,
        Self::Gps,
        Self::Imu,
    ];

    #[must_use]
    pub const fn to_filename(&self) -> &'static str {
        match self {
            Self::Altimeter => "ALTIM.LOG",
            Self::Gps => "GPS.LOG",
            Self::Imu => "IMU.LOG",
        }
    }
}

pub trait LogMessage: telemetry_messages::Serialize {
    const KIND: LogDataType;
}

impl LogMessage for telemetry_messages::AltimeterMessage {
    const KIND: LogDataType = LogDataType::Altimeter;
}

impl LogMessage for telemetry_messages::GpsMessage {
    const KIND: LogDataType = LogDataType::Gps;
}

impl LogMessage for telemetry_messages::ImuMessage {
    const KIND: LogDataType = LogDataType::Imu;
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

pub type Filename = &'static str;

pub trait FileSystem {
    type File;
    type Error: core::fmt::Debug;

    fn exist_file(&mut self, filename: Filename) -> Result<bool, Self::Error>;
    fn create_file(&mut self, filename: Filename) -> Result<Self::File, Self::Error>;

    fn open_file_append(&mut self, filename: Filename) -> Result<Self::File, Self::Error>;
    fn close_file(&mut self, file: Self::File) -> Result<(), Self::Error>;

    fn write_file(&mut self, file: &mut Self::File, data: &[u8]) -> Result<(), Self::Error>;
    fn flush_file(&mut self, file: &mut Self::File) -> Result<(), Self::Error>;
}
