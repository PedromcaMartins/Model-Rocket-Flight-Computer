use defmt::Format;

use crate::{Serialize, Deserialize, Schema};

#[derive(Serialize, Deserialize, Schema, Format, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogDataType {
    Altimeter,
    Gps,
    Imu,
}

impl LogDataType {
    pub const LENGTH: usize = 3;
    pub const MAX_BASE_FILENAME_LENGTH: usize = 3;

    pub const VALUES: [Self; Self::LENGTH] = [
        Self::Altimeter,
        Self::Gps,
        Self::Imu,
    ];

    #[must_use]
    pub const fn to_base_filename(&self) -> &'static str {
        match self {
            Self::Altimeter => "ALT",
            Self::Gps => "GPS",
            Self::Imu => "IMU",
        }
    }

    #[allow(dead_code)]
    const ASSERT_FILENAME_LENGTHS: () = {
        assert!(Self::VALUES[0].to_base_filename().len() <= Self::MAX_BASE_FILENAME_LENGTH);
        assert!(Self::VALUES[2].to_base_filename().len() <= Self::MAX_BASE_FILENAME_LENGTH);
        assert!(Self::VALUES[1].to_base_filename().len() <= Self::MAX_BASE_FILENAME_LENGTH);
    };
}

pub trait LogMessage: Serialize {
    const KIND: LogDataType;
}
