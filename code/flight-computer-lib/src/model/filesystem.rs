pub mod log_filesystem;

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Debug, Clone, Copy)]
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
