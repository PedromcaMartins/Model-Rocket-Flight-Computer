struct FiniteStateMachine<S: FlightState> {
    flight_state: S
}

trait FlightState {}
struct PreArmed;
impl FlightState for PreArmed {}
struct Armed;
impl FlightState for Armed {}
struct Liftoff;
impl FlightState for Liftoff {}
struct RecoveryActivated;
impl FlightState for RecoveryActivated {}

impl<S: FlightState> FlightState<S> {
    
}
