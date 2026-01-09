use core::fmt::Debug;

use embedded_sdmmc::{Mode, RawDirectory, RawFile, VolumeManager};

use crate::interfaces::FileSystem;

pub struct DummyTimeSource;
impl embedded_sdmmc::TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp::from_calendar(2025, 1, 1, 1, 1, 1).expect("dummy time source uses static but valid timestamp")
    }
}

#[defmt_or_log::maybe_derive_format]
#[derive(thiserror::Error, Debug, Clone)]
pub enum SdCardError<E: Debug> {
    #[error("FileSystem error")]
    FileSystem(#[from] embedded_sdmmc::Error<E>),
    #[error("Serialize error")]
    Serialize(#[from] serde_json_core::ser::Error),
}

pub struct SdCardFatFS<
    D,
    const MAX_DIRS: usize = 1,
    const MAX_FILES: usize = 1,
    const MAX_VOLUMES: usize = 1,
> where
    D: embedded_sdmmc::BlockDevice,
{
    volume_manager: VolumeManager<D, DummyTimeSource, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    raw_root_dir: RawDirectory,
}

impl<D, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>
    SdCardFatFS<D, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    D: embedded_sdmmc::BlockDevice,
{
    pub fn init<const ID_OFFSET: u32>(sd_card: D) -> Result<Self, SdCardError<D::Error>> {
        let volume_manager = VolumeManager::new_with_limits(sd_card, DummyTimeSource, ID_OFFSET);
        let raw_volume = volume_manager.open_raw_volume(embedded_sdmmc::VolumeIdx(0))?;
        let raw_root_dir = volume_manager.open_root_dir(raw_volume)?;

        Ok(Self {
            volume_manager,
            raw_root_dir,
        })
    }
}

impl<D: embedded_sdmmc::BlockDevice> FileSystem for SdCardFatFS<D> {
    type File = RawFile;
    type Error = SdCardError<D::Error>;

    async fn exist_file(&mut self, filename: &str) -> Result<bool, Self::Error> {
        match self.volume_manager.find_directory_entry(self.raw_root_dir, filename) {
            Ok(_) => Ok(true),
            Err(embedded_sdmmc::Error::NotFound) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    async fn create_file(&mut self, filename: &str) -> Result<Self::File, Self::Error> {
        self.volume_manager.open_file_in_dir(
            self.raw_root_dir,
            filename,
            Mode::ReadWriteCreate,
        ).map_err(SdCardError::FileSystem)
    }

    async fn open_file_append(&mut self, filename: &str) -> Result<Self::File, Self::Error> {
        self.volume_manager.open_file_in_dir(
            self.raw_root_dir,
            filename,
            Mode::ReadWriteAppend,
        ).map_err(SdCardError::FileSystem)
    }

    async fn close_file(&mut self, file: Self::File) -> Result<(), Self::Error> {
        self.volume_manager.close_file(file).map_err(SdCardError::FileSystem)
    }

    async fn write_file(&mut self, file: &mut Self::File, data: &[u8]) -> Result<(), Self::Error> {
        self.volume_manager.write(
            *file, 
            data,
        ).map_err(SdCardError::FileSystem)
    }

    async fn flush_file(&mut self, file: &mut Self::File) -> Result<(), Self::Error> {
        self.volume_manager.flush_file(
            *file
        ).map_err(SdCardError::FileSystem)
    }
}
