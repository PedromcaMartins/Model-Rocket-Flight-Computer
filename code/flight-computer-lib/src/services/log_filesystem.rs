#[allow(unused_imports)]
use core::fmt::Write as _;
#[allow(unused_imports)]
use embedded_io::Write as _;

use crate::{config::LogFileSystemConfig, events::filesystem::{FileSystemError, FileSystemResult, FileSystemSuccess}, interfaces::{FileSystem, Filename}, services::trace::TraceSync};

use defmt_or_log::{error, Debug2Format};
use heapless::index_map::FnvIndexMap;
use static_cell::ConstStaticCell;
use telemetry_messages::{LogDataType, LogMessage};

type FileUniqueId = u16;
fn append_unique_id(base: &str, uid: FileUniqueId) -> Result<heapless::String<{ LogFileSystemConfig::MAX_FILENAME_LENGTH }>, FileSystemError>  {
    let mut filename = heapless::String::<{ LogFileSystemConfig::MAX_FILENAME_LENGTH }>::new();
    filename.write_fmt(format_args!("{uid:0width$}{base}", width = LogFileSystemConfig::MAX_UID_LENGTH))
        .map_err(|_| FileSystemError::FilenameTooLong)?;
    Ok(filename)
}

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
        static WRITE_BUFFER: ConstStaticCell<[u8; LogFileSystemConfig::WRITE_BUFFER_SIZE]> = ConstStaticCell::new([0_u8; LogFileSystemConfig::WRITE_BUFFER_SIZE]);

        Self {
            file_system,
            files: FnvIndexMap::new(),
            write_buffer: WRITE_BUFFER.take(),
        }
    }

    fn get_unique_id(&mut self, base: Filename) -> Result<Option<FileUniqueId>, FileSystemError> {
        let trace = TraceSync::start("LogFileSystem::get_unique_id");

        for uid in 0..=FileUniqueId::MAX {
            let full_filename = append_unique_id(base, uid)
                .map_err(|_| FileSystemError::FilenameTooLong)?;
            match self.file_system.exist_file(&full_filename) {
                Ok(true) => (),
                Ok(false) => return Ok(Some(uid)),
                Err(e) => {
                    error!("Failed to check existence of file {}: {:?}", full_filename, Debug2Format(&e));
                    return Err(FileSystemError::GetUniqueIdFailed);
                },
            }
        }

        drop(trace);
        Ok(None)
    }

    fn create_file(&mut self, filename: Filename, data_type: LogDataType) -> Result<(), FileSystemError> {
        let file = match self.file_system.create_file(filename) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to create file {}: {:?}", filename, Debug2Format(&e));
                return Err(FileSystemError::FileCreationFailed(data_type));
            },
        };

        match self.files.insert(data_type, file) {
            Ok(None) => Ok(()),
            Ok(Some(_)) => {
                error!("Existing file handle for {} already in the new open files hash map", filename);
                Err(FileSystemError::FileHandleAlreadyExists(data_type))
            },
            Err(_) => {
                error!("Failed to store file handle for {}", filename);
                Err(FileSystemError::StoreFileHandleFailed(data_type))
            }
        }
    }

    pub fn create_unique_files(&mut self) -> FileSystemResult {
        let trace = TraceSync::start("LogFileSystem::create_unique_files");

        let reference_data_type = LogDataType::VALUES[0];
        let uid = self.get_unique_id(reference_data_type.to_base_filename())?.ok_or_else(|| {
            error!("No unique ID available");
            FileSystemError::UniqueIdUnavailable
        })?;

        for data_type in LogDataType::VALUES {
            let base = data_type.to_base_filename();
            self.create_file(
                &append_unique_id(base, uid)?, 
                data_type
            )?;
        }

        drop(trace);
        Ok(FileSystemSuccess::UniqueFilesCreated)
    }

    /// Append a message to the appropriate file based on its type.
    pub fn append_message<M: LogMessage>(&mut self, data: &M) -> FileSystemResult {
        let trace = TraceSync::start("LogFileSystem::append_message");

        let data_type = M::KIND;

        let file = self.files.get_mut(&M::KIND).ok_or_else(|| {
            error!("File for {:?} not opened", M::KIND);
            FileSystemError::FileHandleNotFound(data_type)
        })?;

        let len = serde_json_core::to_slice(&data, self.write_buffer).map_err(|_| {
            error!("Failed to serialize message of type {:?}", M::KIND);
            FileSystemError::FailedToSerializeMessage(data_type)
        })?;

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

        drop(trace);
        Ok(FileSystemSuccess::MessageAppended(M::KIND))
    }

    /// Flush all open files.
    pub fn flush_all(&mut self) -> FileSystemResult {
        let trace = TraceSync::start("LogFileSystem::flush_all");

        for (data_type, file) in &mut self.files {
            if let Err(err) = self.file_system.flush_file(file) {
                error!("Failed to flush file for {:?}: {:?}", data_type, Debug2Format(&err));
                return Err(FileSystemError::FailedToFlushFile(*data_type));
            }
        }

        drop(trace);
        Ok(FileSystemSuccess::FilesFlushed)
    }
}
