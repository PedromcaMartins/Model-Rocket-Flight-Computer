use crate::{Serialize, Deserialize, Schema};

use derive_more::Display;


/* ------------------------------ Flight State ------------------------------ */

#[defmt_or_log_macros::maybe_derive_format]
#[derive(Serialize, Deserialize, Schema, Clone, Copy, Debug, PartialEq, Eq, Default, Display)]
pub enum FlightState {
    #[default]
    PreArmed,
    Armed,
    RecoveryActivated,
    Touchdown,
}
