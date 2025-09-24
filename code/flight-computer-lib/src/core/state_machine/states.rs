mod pre_armed;
mod armed;
mod recovery_activated;
mod touchdown;

pub struct PreArmed;
pub struct Armed;
pub struct RecoveryActivated;
pub struct Touchdown;

pub trait FlightState {}
impl FlightState for PreArmed {}
impl FlightState for Armed {}
impl FlightState for RecoveryActivated {}
impl FlightState for Touchdown {}
