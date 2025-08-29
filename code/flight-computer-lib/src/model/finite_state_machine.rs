use defmt_or_log::info;
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use uom::si::{f64::Length, length::meter};

pub struct PreArmed<M>
where
    M: RawMutex + 'static,
{
    arm_button_pushed_signal: &'static Signal<M, ()>,
}

pub struct Armed<M>
where
    M: RawMutex + 'static,
{
    launchpad_altitude: Length,
    latest_altitude_signal: &'static Signal<M, Length>,
}

pub struct RecoveryActivated<M>
where
    M: RawMutex + 'static,
{
    launchpad_altitude: Length,
    latest_altitude_signal: &'static Signal<M, Length>,
}
pub struct Touchdown;

pub struct FiniteStateMachine<S: FlightState> {
    flight_state: S,
}

pub trait FlightState {}
impl<M: RawMutex> FlightState for PreArmed<M> {}
impl<M: RawMutex> FlightState for Armed<M> {}
impl<M: RawMutex> FlightState for RecoveryActivated<M> {}
impl FlightState for Touchdown {}

impl<M: RawMutex> FiniteStateMachine<PreArmed<M>> {
    pub const fn new(arm_button_pushed_signal: &'static Signal<M, ()>) -> Self {
        Self {
            flight_state: PreArmed {
                arm_button_pushed_signal,
            },
        }
    }
}

impl<M: RawMutex> FiniteStateMachine<PreArmed<M>>
{
    pub async fn wait_arm(self, latest_altitude_signal: &'static Signal<M, Length>) -> FiniteStateMachine<Armed<M>> {
        self.flight_state.arm_button_pushed_signal.wait().await;

        info!("Flight Computer Armed");

        let launchpad_altitude = latest_altitude_signal.wait().await;

        info!("Launchpad Altitude: {} m", launchpad_altitude.get::<meter>());

        FiniteStateMachine {
            flight_state: Armed {
                launchpad_altitude,
                latest_altitude_signal,
            },
        }
    }
}

impl<M: RawMutex> FiniteStateMachine<Armed<M>>
{
    pub async fn wait_activate_recovery(self) -> FiniteStateMachine<RecoveryActivated<M>> {
        use uom::si::length::meter;

        loop {
            let altitude = self.flight_state.latest_altitude_signal.wait().await;
            let launchpad_altitude = self.flight_state.launchpad_altitude;
            let altitude = altitude - launchpad_altitude;

            let min_altitude_deployment = Length::new::<meter>(2.0);

            if altitude > min_altitude_deployment {
                info!("Apogee of {} m Reached!", altitude.get::<meter>());
                break;
            }
        }

        info!("Recovery Activated");

        FiniteStateMachine {
            flight_state: RecoveryActivated {
                launchpad_altitude: self.flight_state.launchpad_altitude,
                latest_altitude_signal: self.flight_state.latest_altitude_signal,
            },
        }
    }
}

impl<M: RawMutex> FiniteStateMachine<RecoveryActivated<M>>
{
    pub async fn wait_touchdown(self) -> FiniteStateMachine<Touchdown> {
        use uom::si::length::meter;

        loop {
            let altitude = self.flight_state.latest_altitude_signal.wait().await;
            let launchpad_altitude = self.flight_state.launchpad_altitude;
            let altitude = altitude - launchpad_altitude;

            let min_altitude_deployment = Length::new::<meter>(2.0);

            if altitude <= min_altitude_deployment && altitude <= launchpad_altitude {
                break;
            }
        }

        info!("Touchdown!");

        FiniteStateMachine {
            flight_state: Touchdown,
        }
    }
}
