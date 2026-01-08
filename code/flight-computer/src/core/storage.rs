#[allow(unused_imports)]
use core::fmt::Write as _;
#[allow(unused_imports)]
use embedded_io::Write as _;
use proto::{Record, error::FileSystemError};

use crate::{config::StorageConfig, interfaces::FileSystem, core::trace::TraceSync};

use defmt_or_log::{Debug2Format, error};
use static_cell::ConstStaticCell;
type FileUniqueId = u16;

pub struct Storage<FS, FH>
where
    FS: FileSystem<File = FH>,
{
    filesystem: FS,
    file: FH,
    write_buffer: &'static mut [u8],
}

impl<FS, FH> Storage<FS, FH>
where
    FS: FileSystem<File = FH>,
{
    pub async fn new(mut filesystem: FS) -> Result<Self, FileSystemError> {
        static WRITE_BUFFER: ConstStaticCell<[u8; StorageConfig::WRITE_BUFFER_SIZE]> = ConstStaticCell::new([0_u8; StorageConfig::WRITE_BUFFER_SIZE]);
        let mut filename: heapless::String<{ StorageConfig::MAX_FILENAME_LENGTH }> = heapless::String::new();
        let trace = TraceSync::start("Storage::new");

        for uid in 0..=FileUniqueId::MAX {
            write!(&mut filename, "{uid}").map_err(|_| FileSystemError::FilenameTooLong)?;
            match filesystem.exist_file(&filename).await {
                Err(e) => {
                    error!("Failed to check existence of file {}: {:?}", filename, Debug2Format(&e));
                    return Err(FileSystemError::GetUniqueIdFailed);
                },
                Ok(true) if uid == FileUniqueId::MAX => {           
                    error!("No unique ID available");
                    return Err(FileSystemError::UniqueIdUnavailable);
                },
                Ok(true) => (),
                Ok(false) => break,
            }
        }

        let file = match filesystem.create_file(&filename).await {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to create file {}: {:?}", filename, Debug2Format(&e));
                return Err(FileSystemError::FileCreationFailed);
            },
        };

        drop(trace);
        Ok(Self {
            filesystem,
            file,
            write_buffer: WRITE_BUFFER.take(),
        })
    }

    pub async fn append_record(&mut self, data: &Record) -> Result<(), FileSystemError> {
        let trace = TraceSync::start("Storage::append_record");

        let len = serde_json_core::to_slice(&data, self.write_buffer).map_err(|_| {
            error!("Failed to serialize record");
            FileSystemError::FailedToSerializeRecord
        })?;

        // Add newline sequence to the buffer
        if len + 2 > self.write_buffer.len() {
            error!("Buffer too small to add newline to buffer");
            return Err(FileSystemError::WriteBufferTooSmall);
        }
        self.write_buffer[len] = b'\r';
        self.write_buffer[len + 1] = b'\n';
        let len = len + 2;

        self.filesystem.write_file(&mut self.file, &self.write_buffer[..len]).await.map_err(|e| {
            error!("Failed to append data: {:?}", Debug2Format(&e));
            FileSystemError::FailedToWriteRecord
        })?;

        drop(trace);
        Ok(())
    }

    /// Flush all open files.
    pub async fn flush(&mut self) -> Result<(), FileSystemError> {
        let trace = TraceSync::start("Storage::flush");

        self.filesystem.flush_file(&mut self.file).await.map_err(|e| {
            error!("Failed to flush file:{:?}", Debug2Format(&e));
            FileSystemError::FailedToFlushFile
        })?;

        drop(trace);
        Ok(())
    }
}
