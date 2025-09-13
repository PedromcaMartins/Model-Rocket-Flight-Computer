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

    pub const VALUES: [Self; Self::LENGTH] = [
        Self::Altimeter,
        Self::Gps,
        Self::Imu,
    ];

    #[must_use]
    pub const fn to_filename(&self) -> &'static str {
        match self {
            Self::Altimeter => "ALTIM.LOG",
            Self::Gps => "GPS.LOG",
            Self::Imu => "IMU.LOG",
        }
    }
}

pub trait LogMessage: Serialize {
    const KIND: LogDataType;
}
