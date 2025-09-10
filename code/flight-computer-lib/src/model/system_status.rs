use core::num::Saturating;

use embassy_time::Instant;
use enum_map::EnumMap;

use crate::model::filesystem::{FileSystemEvent, LogDataType};

#[macro_export]
macro_rules! send_to_system_status {
    ($channel:expr, $err_counter:expr, $expr:expr) => {
        if $channel.try_send(Ok($expr)).is_err() {
            $err_counter += 1;
        }
    };
}

#[macro_export]
macro_rules! error_sending_to_system_status {
    ($channel:expr, $err_counter:expr) => {
        if $channel.try_send(Err($err_counter.0)).is_err() {
            $err_counter += 1;
        }
    };
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Debug, Clone, Copy, enum_map::Enum)]
pub enum ArmButtonSystemStatus {
    ArmButtonPressed,
    FailedToReadArmButton,
    FailedToSendChannel,
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Debug, Clone, Copy, enum_map::Enum)]
pub enum AltimeterSystemStatus {
    MessageParsed,
    FailedToInitializeDevice,
    FailedToParseMessage,
    FailedToPublishToPostcard,
    FailedToPublishToSdCard,
    FailedToSendChannel,
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Debug, Clone, Copy, enum_map::Enum)]
pub enum ImuSystemStatus {
    MessageParsed,
    FailedToInitializeDevice,
    FailedToParseMessage,
    FailedToPublishToPostcard,
    FailedToPublishToSdCard,
    FailedToSendChannel,
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Debug, Clone, Copy, enum_map::Enum)]
pub enum GpsSystemStatus {
    MessageParsed,
    FailedToInitializeDevice,
    FailedToParseMessage,
    FailedToPublishToPostcard,
    FailedToPublishToSdCard,
    FailedToSendChannel,
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Debug, Clone, enum_map::Enum)]
pub enum SdCardSystemStatus {
    FileSystemEvent(LogDataType, FileSystemEvent),
    FailedToSendChannel,
    Other,
}

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, enum_map::Enum)]
pub enum FlightState {
    PreArmed,
    Armed,
    RecoveryActivated,
    Touchdown,
}

#[derive(Debug, Default)]
pub struct SystemStatus {
    arm_button: EnumMap<ArmButtonSystemStatus, Saturating<usize>>,
    altimeter: EnumMap<AltimeterSystemStatus, Saturating<usize>>,
    imu: EnumMap<ImuSystemStatus, Saturating<usize>>,
    gps: EnumMap<GpsSystemStatus, Saturating<usize>>,
    sd_card: EnumMap<SdCardSystemStatus, Saturating<usize>>,
    finite_state_machine: EnumMap<FlightState, Option<Instant>>,
}

impl SystemStatus {
    pub fn update_arm_button_status(&mut self, status: Result<ArmButtonSystemStatus, usize>) {
        match status {
            Ok(status) => self.arm_button[status] += 1,
            Err(attempts) => self.arm_button[ArmButtonSystemStatus::FailedToSendChannel] += attempts,
        }
    }

    pub fn update_altimeter_status(&mut self, status: Result<AltimeterSystemStatus, usize>) {
        match status {
            Ok(status) => self.altimeter[status] += 1,
            Err(attempts) => self.altimeter[AltimeterSystemStatus::FailedToSendChannel] += attempts,
        }
    }

    pub fn update_imu_status(&mut self, status: Result<ImuSystemStatus, usize>) {
        match status {
            Ok(status) => self.imu[status] += 1,
            Err(attempts) => self.imu[ImuSystemStatus::FailedToSendChannel] += attempts,
        }
    }

    pub fn update_gps_status(&mut self, status: Result<GpsSystemStatus, usize>) {
        match status {
            Ok(status) => self.gps[status] += 1,
            Err(attempts) => self.gps[GpsSystemStatus::FailedToSendChannel] += attempts,
        }
    }

    pub fn update_sd_card_status(&mut self, status: Result<SdCardSystemStatus, usize>) {
        match status {
            Ok(status) => self.sd_card[status] += 1,
            Err(attempts) => self.sd_card[SdCardSystemStatus::FailedToSendChannel] += attempts,
        }
    }

    pub fn update_finite_state_machine_status(&mut self, status: FlightState) {
        self.finite_state_machine[status] = Some(Instant::now());
    }
}
