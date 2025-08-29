use core::fmt::Debug;

use embedded_sdmmc::{filesystem::ToShortFileName, Mode, RawDirectory, RawFile, RawVolume, VolumeManager};
use enum_map::{enum_map, Enum, EnumMap};

use crate::model::filesystem::{FileSystem, FileSystemEvent, LogDataType};

pub struct DummyTimeSource;
impl embedded_sdmmc::TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp::from_calendar(2025, 1, 1, 1, 1, 1).expect("dummy time source uses valid timestamp")
    }
}

#[defmt_or_log::maybe_derive_format]
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

pub struct LogFile<N: ToShortFileName + Clone> {
    pub filename: N,
    pub file: RawFile,
}

pub struct SdCardDevice<W, O, D, N>
where
    W: switch_hal::WaitSwitch<Error: Debug>,
    O: switch_hal::OutputSwitch<Error: Debug>,
    D: embedded_sdmmc::BlockDevice,
    N: embedded_sdmmc::filesystem::ToShortFileName + Clone,
{
    sd_card_detect: W,
    sd_card_status_led: O,

    volume_manager: VolumeManager<D, DummyTimeSource, 1, { LogDataType::LENGTH }, 1>,
    raw_volume: RawVolume,
    raw_directory: RawDirectory,

    files: EnumMap<LogDataType, LogFile<N>>,
}

impl<W, O, D, N> SdCardDevice<W, O, D, N>
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
        let volume_manager = VolumeManager::new_with_limits(sd_card, DummyTimeSource, ID_OFFSET);
        let raw_volume = volume_manager.open_raw_volume(embedded_sdmmc::VolumeIdx(0))?;
        let raw_directory = volume_manager.open_root_dir(raw_volume)?;

        let altimeter_file = volume_manager.open_file_in_dir(raw_directory, altimeter_filename.clone(), Mode::ReadWriteAppend)?;
        let gps_file = volume_manager.open_file_in_dir(raw_directory, gps_filename.clone(), Mode::ReadWriteAppend)?;
        let imu_file = volume_manager.open_file_in_dir(raw_directory, imu_filename.clone(), Mode::ReadWriteAppend)?;

        let files = enum_map! {
            LogDataType::Altimeter => LogFile { filename: altimeter_filename.clone(), file: altimeter_file },
            LogDataType::Gps => LogFile { filename: gps_filename.clone(), file: gps_file },
            LogDataType::Imu => LogFile { filename: imu_filename.clone(), file: imu_file },
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

    fn activate_status_led(&mut self) -> Result<(), SdCardError<D::Error>> {
        self.sd_card_status_led.on().map_err(|_| SdCardError::SetStatusLed)
    }

    fn deactivate_status_led(&mut self) -> Result<(), SdCardError<D::Error>> {
        self.sd_card_status_led.off().map_err(|_| SdCardError::SetStatusLed)
    }

    fn append_message<T: telemetry_messages::Serialize>(
        &mut self,
        file_data: LogDataType,
        message: &T,
        buffer: &mut [u8]
    ) -> Result<(), SdCardError<D::Error>> {
        self.activate_status_led()?;

        let entry = &mut self.files[file_data];
        let len = serde_json_core::to_slice(message, buffer)?;
        self.volume_manager.write(entry.file, &buffer[..len])?;

        self.deactivate_status_led()?;
        Ok(())
    }

    fn flush_file(&mut self, log_data_type: LogDataType) -> Result<(), SdCardError<D::Error>> {
        self.activate_status_led()?;

        let entry = &mut self.files[log_data_type];
        self.volume_manager.flush_file(entry.file)?;

        self.deactivate_status_led()?;
        Ok(())
    }

    fn reopen_file(&mut self, log_data_type: LogDataType) -> Result<(), SdCardError<D::Error>> {
        self.activate_status_led()?;

        let entry = &mut self.files[log_data_type];
        self.volume_manager.close_file(entry.file)?;
        let filename = entry.filename.clone();
        entry.file = self.volume_manager.open_file_in_dir(self.raw_directory, filename, Mode::ReadWriteAppend)?;

        self.deactivate_status_led()?;
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

impl<W, O, D, N> FileSystem for SdCardDevice<W, O, D, N>
where
    W: switch_hal::WaitSwitch<Error: Debug>,
    O: switch_hal::OutputSwitch<Error: Debug>,
    D: embedded_sdmmc::BlockDevice,
    N: embedded_sdmmc::filesystem::ToShortFileName + Clone,
{
    fn append_message<T: telemetry_messages::Serialize>(
        &mut self,
        file_data: LogDataType,
        message: &T,
        buffer: &mut [u8]
    ) -> FileSystemEvent {
        match self.append_message(file_data, message, buffer) {
            Ok(()) => FileSystemEvent::MessageWritten,
            Err(SdCardError::Serialize(_)) => FileSystemEvent::FailedToSerializeMessage,
            Err(SdCardError::FileSystem(_)) => FileSystemEvent::FailedToWriteMessage,
            Err(_) => FileSystemEvent::Other,
        }
    }

    fn flush_file(&mut self, log_data_type: LogDataType) -> FileSystemEvent {
        match self.flush_file(log_data_type) {
            Ok(()) => FileSystemEvent::FileFlushed,
            Err(SdCardError::FileSystem(_)) => FileSystemEvent::FailedToFlushFile,
            Err(_) => FileSystemEvent::Other,
        }
    }

    fn reopen_file(&mut self, log_data_type: LogDataType) -> FileSystemEvent {
        match self.reopen_file(log_data_type) {
            Ok(()) => FileSystemEvent::FileReopened,
            Err(SdCardError::FileSystem(_)) => FileSystemEvent::FailedToReopenFile,
            Err(_) => FileSystemEvent::Other,
        }
    }
}
