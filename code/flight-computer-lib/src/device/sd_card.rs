use core::fmt::Debug;

use embedded_sdmmc::{filesystem::ToShortFileName, Mode, RawDirectory, RawFile, RawVolume, VolumeManager};
use enum_map::{enum_map, Enum, EnumMap};

pub struct DummyTimeSource;
impl embedded_sdmmc::TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp::from_calendar(2025, 1, 1, 1, 1, 1).expect("dummy time source uses valid timestamp")
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum SdCardError<E: Debug> {
    #[error("Init: Sd Card not recognized")]
    SdCardNotRecognized,
    #[error("FileSystem error")]
    FileSystem(#[from] embedded_sdmmc::Error<E>),
    #[error("Serialize error")]
    Serialize(#[from] serde_json_core::ser::Error),
    #[error("SD Card Status LED set error")]
    SetStatusLed,
}

#[derive(Enum)]
pub enum LogFileData {
    Altimeter,
    Gps,
    Imu,
}

pub struct LogFile<N: ToShortFileName + Clone> {
    pub filename: N,
    pub file: RawFile,
}

pub struct SdCardDevice<
    W, O, D, N, 
    const MAX_DIRS: usize, 
    const MAX_FILES: usize, 
    const MAX_VOLUMES: usize, 
>
where
    W: switch_hal::WaitSwitch<Error: Debug>,
    O: switch_hal::OutputSwitch<Error: Debug>,
    D: embedded_sdmmc::BlockDevice,
    N: embedded_sdmmc::filesystem::ToShortFileName + Clone,
{
    sd_card_detect: W,
    sd_card_status_led: O,

    volume_manager: VolumeManager<D, DummyTimeSource, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    raw_volume: RawVolume,
    raw_directory: RawDirectory,

    files: EnumMap<LogFileData, LogFile<N>>,
}

impl<
    W, O, D, N, 
    const MAX_DIRS: usize, 
    const MAX_FILES: usize, 
    const MAX_VOLUMES: usize, 
> SdCardDevice<W, O, D, N, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    W: switch_hal::WaitSwitch<Error: Debug>,
    O: switch_hal::OutputSwitch<Error: Debug>,
    D: embedded_sdmmc::BlockDevice,
    N: embedded_sdmmc::filesystem::ToShortFileName + Clone,
{
    pub fn init<const ID_OFFSET: u32> (
        sd_card: D,
        sd_card_detect: W,
        sd_card_status_led: O,

        altimeter_filename: &N,
        gps_filename: &N,
        imu_filename: &N,
    ) -> Result<Self, SdCardError<D::Error>> {
        let volume_manager: VolumeManager<D, DummyTimeSource, MAX_DIRS, MAX_FILES, MAX_VOLUMES>= VolumeManager::new_with_limits(sd_card, DummyTimeSource, ID_OFFSET);
        let raw_volume = volume_manager.open_raw_volume(embedded_sdmmc::VolumeIdx(0))?;
        let raw_directory = volume_manager.open_root_dir(raw_volume)?;

        let altimeter_file = volume_manager.open_file_in_dir(raw_directory, altimeter_filename.clone(), Mode::ReadWriteAppend)?;
        let gps_file = volume_manager.open_file_in_dir(raw_directory, gps_filename.clone(), Mode::ReadWriteAppend)?;
        let imu_file = volume_manager.open_file_in_dir(raw_directory, imu_filename.clone(), Mode::ReadWriteAppend)?;

        let files = enum_map! {
            LogFileData::Altimeter => LogFile { filename: altimeter_filename.clone(), file: altimeter_file },
            LogFileData::Gps => LogFile { filename: gps_filename.clone(), file: gps_file },
            LogFileData::Imu => LogFile { filename: imu_filename.clone(), file: imu_file },
        };

        Ok(Self {
            sd_card_detect,
            sd_card_status_led,

            volume_manager,
            raw_volume,
            raw_directory,

            files,
        })
    }

    pub fn append_message<T: telemetry_messages::Serialize>(
        &mut self,
        file_data: LogFileData,
        message: &T,
        buffer: &mut [u8],
    ) -> Result<(), SdCardError<D::Error>> {
        self.sd_card_status_led.on().map_err(|_| SdCardError::SetStatusLed)?;

        let file = self.files[file_data].file;
        let len = serde_json_core::to_slice(message, buffer)?;
        self.volume_manager.write(file, &buffer[..len])?;

        self.sd_card_status_led.off().map_err(|_| SdCardError::SetStatusLed)?;
        Ok(())
    }

    pub fn flush_all_files(&mut self) -> Result<(), SdCardError<D::Error>> {
        self.sd_card_status_led.on().map_err(|_| SdCardError::SetStatusLed)?;

        for entry in self.files.values() {
            self.volume_manager.flush_file(entry.file)?;
        }

        self.sd_card_status_led.off().map_err(|_| SdCardError::SetStatusLed)?;
        Ok(())
    }

    pub fn reopen_all_files(&mut self) -> Result<(), SdCardError<D::Error>> {
        self.sd_card_status_led.on().map_err(|_| SdCardError::SetStatusLed)?;

        for entry in self.files.values_mut() {
            self.volume_manager.close_file(entry.file)?;
            let filename = entry.filename.clone();
            entry.file = self.volume_manager.open_file_in_dir(self.raw_directory, filename, Mode::ReadWriteAppend)?;
        }

        self.sd_card_status_led.off().map_err(|_| SdCardError::SetStatusLed)?;
        Ok(())
    }

    pub fn destroy(self) -> Result<(D, W, O), SdCardError<D::Error>> {
        for entry in self.files.values() {
            self.volume_manager.close_file(entry.file)?;
        }
        self.volume_manager.close_dir(self.raw_directory)?;
        self.volume_manager.close_volume(self.raw_volume)?;
        Ok((self.volume_manager.free().0, self.sd_card_detect, self.sd_card_status_led))
    }
}
