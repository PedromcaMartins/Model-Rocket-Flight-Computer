use crate::{Serialize, Deserialize, Schema};


/* ----------------------------- File System Led ---------------------------- */

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LedStatus {
    On,
    #[default]
    Off,
}

/* ----------------------------- Actuator Status ---------------------------- */

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ActuatorStatus {
    Active,
    #[default]
    Inactive,
}
