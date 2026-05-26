// Wire transport types — only adapter code should import from this module.
// Domain code uses proto::sensor_data::*, proto::flight_state::*, etc.

pub use crate::record::{Record, RecordData};
pub use crate::record::tick_hz::GlobalTickHz;
pub use crate::record::uid::Uid;
pub use crate::record::tick_hz::Timestamp;

#[cfg(feature = "transport-ipc")]
pub use crate::transport::ipc::*;

// Postcard-rpc Endpoints
pub use crate::{ENDPOINT_LIST, PingEndpoint, GlobalTickHzEndpoint};

// GS-facing Topics
pub use crate::{TOPICS_GS_IN_LIST, TOPICS_GS_OUT_LIST, RecordTopic};

// Simulator-facing Topics (cfg-gated)
#[cfg(feature = "simulator-endpoints")]
pub use crate::{
    TOPICS_SIM_IN_LIST, TOPICS_SIM_OUT_LIST,
    SimAltimeterTopic, SimGpsTopic, SimImuTopic, SimArmTopic,
    SimDeploymentTopic, SimFlightStateTopic,
    SimPostcardLedTopic, SimAltimeterLedTopic, SimGpsLedTopic,
    SimImuLedTopic, SimArmLedTopic, SimFileSystemLedTopic,
    SimDeploymentLedTopic, SimGroundStationLedTopic,
};
