use crate::model::filesystem::{FileSystem, FileSystemEvent, LogDataType, LogMessage};
use enum_map::Enum;
use defmt_or_log::error;
use static_cell::ConstStaticCell;

pub struct LogFileSystem<FS, FH>
where
    FS: FileSystem<File = FH>,
{
    file_system: FS,
    files: heapless::FnvIndexMap<LogDataType, FH, { LogDataType::LENGTH }>,
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
            files: heapless::FnvIndexMap::new(),
            write_buffer: WRITE_BUFFER.take(),
        }
    }

    /// Change directory to unique id folder.
    pub fn create_unique_files(&mut self) -> FileSystemEvent {
        for data_type in LogDataType::VALUES {
            let filename = data_type.to_filename();
            match self.file_system.create_file(filename) {
                Ok(file) => { 
                    match self.files.insert(data_type, file) {
                        Ok(None) => (),
                        Ok(Some(_)) => {
                            error!("Existing file handle for {} already in the open files hash map", filename);
                            return FileSystemEvent::Other;
                        }, 
                        Err(_) => {
                            error!("Failed to store file handle for {}", filename);
                            return FileSystemEvent::Other;
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to create file {}: {:?}", filename, e);
                    return FileSystemEvent::Other;
                },
            }
        }
        FileSystemEvent::Other
    }

    /// Append a message to the appropriate file based on its type.
    pub fn append_message<M: LogMessage>(&mut self, data: &M) -> FileSystemEvent {
        let Some(file) = self.files.get_mut(&M::KIND) else {
            error!("File for {:?} not opened", M::KIND);
            return FileSystemEvent::Other;
        };

        let Ok(len) = serde_json_core::to_slice(&data, self.write_buffer) else {
            error!("Failed to serialize message of type {:?}", M::KIND);
            return FileSystemEvent::FailedToSerializeMessage;
        };

        if let Err(err) = self.file_system.write_file(file, &self.write_buffer[..len]) {
            error!("Failed to append data: {:?}", err);
            return FileSystemEvent::FailedToWriteMessage;
        }
        FileSystemEvent::Other
    }

    /// Flush all open files.
    pub fn flush_all(&mut self) -> FileSystemEvent {
        for (data_type, file) in &mut self.files {
            if let Err(err) = self.file_system.flush_file(file) {
                error!("Failed to flush file for {:?}: {:?}", data_type, err);
                return FileSystemEvent::FailedToFlushFile;
            }
        }
        FileSystemEvent::FileFlushed
    }
}
