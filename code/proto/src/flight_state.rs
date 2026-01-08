use crate::{Serialize, Deserialize, Schema};


/* ------------------------------ Flight State ------------------------------ */

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlightState {
    #[default]
    PreArmed,
    Armed,
    RecoveryActivated,
    Touchdown,
}
