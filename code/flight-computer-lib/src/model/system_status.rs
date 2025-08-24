use core::num::Saturating;

#[derive(Debug, Clone, Default)]
pub struct ArmButtonSystemStatus {
    pub arm_button_pressed: Saturating<usize>,
    pub failed_to_read_arm_button: Saturating<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct AltimeterSystemStatus {
    pub message_parsed: Saturating<usize>,
    pub failed_to_initialize_device: bool,
    pub failed_to_parse_message: Saturating<usize>,
    pub failed_to_publish_to_postcard: Saturating<usize>,
    pub failed_to_publish_to_sd_card: Saturating<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct ImuSystemStatus {
    pub message_parsed: Saturating<usize>,
    pub failed_to_initialize_device: Saturating<usize>,
    pub failed_to_parse_message: Saturating<usize>,
    pub failed_to_publish_to_postcard: Saturating<usize>,
    pub failed_to_publish_to_sd_card: Saturating<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct GpsSystemStatus {
    pub message_parsed: Saturating<usize>,
    pub failed_to_initialize_device: Saturating<usize>,
    pub failed_to_parse_message: Saturating<usize>,
    pub failed_to_publish_to_postcard: Saturating<usize>,
    pub failed_to_publish_to_sd_card: Saturating<usize>,
}

#[derive(Debug, Clone)]
pub enum FiniteStateMachineStatus {
    PreArmed,
    Armed,
    RecoveryActivated,
    Touchdown,
}

#[derive(Debug, Clone, Default)]
pub struct SdCardSystemStatus {
    pub altimeter_message_written: Saturating<usize>,
    pub gps_message_written: Saturating<usize>,
    pub imu_message_written: Saturating<usize>,
    pub files_flushed: Saturating<usize>,

    pub sd_card_not_recognized: Saturating<usize>,
    pub failed_to_open_volume: Saturating<usize>,
    pub failed_to_open_root_dir: Saturating<usize>,

    pub failed_to_open_altimeter_file: Saturating<usize>,
    pub failed_to_open_gps_file: Saturating<usize>,
    pub failed_to_open_imu_file: Saturating<usize>,

    pub failed_to_serialize_altimeter_msg: Saturating<usize>,
    pub failed_to_serialize_gps_msg: Saturating<usize>,
    pub failed_to_serialize_imu_msg: Saturating<usize>,

    pub failed_to_write_altimeter_msg: Saturating<usize>,
    pub failed_to_write_gps_msg: Saturating<usize>,
    pub failed_to_write_imu_msg: Saturating<usize>,

    pub failed_to_flush_altimeter_file: Saturating<usize>,
    pub failed_to_flush_gps_file: Saturating<usize>,
    pub failed_to_flush_imu_file: Saturating<usize>,
}
