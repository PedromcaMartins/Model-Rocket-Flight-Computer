use crate::{config::LogFileSystemConfig, model::{filesystem::FileSystem, system_event::filesystem::{FileSystemError, FileSystemResult, FileSystemSuccess}}};
use defmt_or_log::{error, Debug2Format};
use heapless::index_map::FnvIndexMap;
use static_cell::ConstStaticCell;
use telemetry_messages::{LogDataType, LogMessage};

pub struct LogFileSystem<FS, FH>
where
    FS: FileSystem<File = FH>,
{
    file_system: FS,
    files: FnvIndexMap<LogDataType, FH, { LogFileSystemConfig::FNV_INDEX_MAP_SIZE }>,
    write_buffer: &'static mut [u8],
}

impl<FS, FH> LogFileSystem<FS, FH>
where
    FS: FileSystem<File = FH>,
{
    pub fn new(file_system: FS) -> Self {
        const WRITE_BUFFER_SIZE: usize = 256;
        static WRITE_BUFFER: ConstStaticCell<[u8; WRITE_BUFFER_SIZE]> = ConstStaticCell::new([0_u8; WRITE_BUFFER_SIZE]);

        Self {
            file_system,
            files: FnvIndexMap::new(),
            write_buffer: WRITE_BUFFER.take(),
        }
    }

    pub fn create_unique_files(&mut self) -> FileSystemResult {
        for data_type in LogDataType::VALUES {
            let filename = data_type.to_filename();
            let file = match self.file_system.create_file(filename) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to create file {}: {:?}", filename, Debug2Format(&e));
                    return Err(FileSystemError::FileCreationFailed(data_type));
                },
            };

            match self.files.insert(data_type, file) {
                Ok(None) => (),
                Ok(Some(_)) => {
                    error!("Existing file handle for {} already in the new open files hash map", filename);
                    return Err(FileSystemError::FileHandleAlreadyExists(data_type));
                },
                Err(_) => {
                    error!("Failed to store file handle for {}", filename);
                    return Err(FileSystemError::StoreFileHandleFailed(data_type));
                }
            }
        }
        Ok(FileSystemSuccess::UniqueFilesCreated)
    }

    /// Append a message to the appropriate file based on its type.
    pub fn append_message<M: LogMessage>(&mut self, data: &M) -> FileSystemResult {
        let data_type = M::KIND;

        let Some(file) = self.files.get_mut(&M::KIND) else {
            error!("File for {:?} not opened", M::KIND);
            return Err(FileSystemError::FileHandleNotFound(data_type));
        };

        let Ok(len) = serde_json_core::to_slice(&data, self.write_buffer) else {
            error!("Failed to serialize message of type {:?}", M::KIND);
            return Err(FileSystemError::FailedToSerializeMessage(data_type));
        };

        // Add newline sequence to the buffer
        if len + 2 > self.write_buffer.len() {
            error!("Buffer too small to add newline for message of type {:?}", M::KIND);
            return Err(FileSystemError::FailedToSerializeMessage(data_type));
        }
        self.write_buffer[len] = b'\r';
        self.write_buffer[len + 1] = b'\n';
        let len = len + 2;

        if let Err(err) = self.file_system.write_file(file, &self.write_buffer[..len]) {
            error!("Failed to append data: {:?}", Debug2Format(&err));
            return Err(FileSystemError::FailedToWriteMessage(data_type));
        }
        Ok(FileSystemSuccess::MessageAppended(M::KIND))
    }

    /// Flush all open files.
    pub fn flush_all(&mut self) -> FileSystemResult {
        for (data_type, file) in &mut self.files {
            if let Err(err) = self.file_system.flush_file(file) {
                error!("Failed to flush file for {:?}: {:?}", data_type, Debug2Format(&err));
                return Err(FileSystemError::FailedToFlushFile(*data_type));
            }
        }
        Ok(FileSystemSuccess::FilesFlushed)
    }
}
