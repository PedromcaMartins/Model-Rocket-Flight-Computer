use defmt_or_log::info;
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use uom::si::{f64::Length, length::meter};

pub struct PreArmed<M>
where
    M: RawMutex + 'static,
{
    arm_button_signal: &'static Signal<M, ()>,
}

pub struct Armed<M>
where
    M: RawMutex + 'static,
{
    launchpad_altitude: Length,
    altitude_signal: &'static Signal<M, Length>,
}

pub struct RecoveryActivated<M>
where
    M: RawMutex + 'static,
{
    launchpad_altitude: Length,
    altitude_signal: &'static Signal<M, Length>,
}
pub struct FlightTerminated;

pub struct FiniteStateMachine<S: FlightState> {
    flight_state: S,
}

pub trait FlightState {}
impl<M: RawMutex> FlightState for PreArmed<M> {}
impl<M: RawMutex> FlightState for Armed<M> {}
impl<M: RawMutex> FlightState for RecoveryActivated<M> {}
impl FlightState for FlightTerminated {}

impl<M: RawMutex> FiniteStateMachine<PreArmed<M>> {
    pub const fn new(arm_button_signal: &'static Signal<M, ()>) -> Self {
        Self {
            flight_state: PreArmed {
                arm_button_signal,
            },
        }
    }
}

impl<M: RawMutex> FiniteStateMachine<PreArmed<M>>
{
    pub async fn wait_arm(self, altitude_signal: &'static Signal<M, Length>) -> FiniteStateMachine<Armed<M>> {
        self.flight_state.arm_button_signal.wait().await;

        info!("Flight Computer Armed");

        let launchpad_altitude = altitude_signal.wait().await;

        info!("Launchpad Altitude: {} m", launchpad_altitude.get::<meter>());

        FiniteStateMachine {
            flight_state: Armed {
                launchpad_altitude,
                altitude_signal,
            },
        }
    }
}

impl<M: RawMutex> FiniteStateMachine<Armed<M>>
{
    pub async fn wait_activate_recovery(self) -> FiniteStateMachine<RecoveryActivated<M>> {
        use uom::si::length::meter;

        loop {
            let altitude = self.flight_state.altitude_signal.wait().await;
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
                altitude_signal: self.flight_state.altitude_signal,
            },
        }
    }
}

impl<M: RawMutex> FiniteStateMachine<RecoveryActivated<M>>
{
    pub async fn wait_touchdown(self) -> FiniteStateMachine<FlightTerminated> {
        use uom::si::length::meter;

        loop {
            let altitude = self.flight_state.altitude_signal.wait().await;
            let launchpad_altitude = self.flight_state.launchpad_altitude;
            let altitude = altitude - launchpad_altitude;

            let min_altitude_deployment = Length::new::<meter>(2.0);

            if altitude <= min_altitude_deployment && altitude <= launchpad_altitude {
                break;
            }
        }

        info!("Touchdown!");

        FiniteStateMachine {
            flight_state: FlightTerminated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_time::{Duration, MockDriver};
    use log::info;
    use rstest::{fixture, rstest};
    use futures::join;
    extern crate std;

    #[fixture]
    fn arm_button_signal() -> &'static Signal<CriticalSectionRawMutex, ()> {
        static SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
        &SIGNAL
    }

    #[fixture]
    fn altitude_signal() -> &'static Signal<CriticalSectionRawMutex, Length> {
        static SIGNAL: Signal<CriticalSectionRawMutex, Length> = Signal::new();
        &SIGNAL
    }

    #[fixture]
    fn altitudes() -> (std::vec::Vec<Length>, Duration) {
        let altitudes = std::vec![
            Length::new::<meter>(1.0),
            Length::new::<meter>(1.0),
            Length::new::<meter>(1.0),
            Length::new::<meter>(2.0),
            Length::new::<meter>(3.0),
            Length::new::<meter>(4.0),
            Length::new::<meter>(3.0),
            Length::new::<meter>(2.0),
            Length::new::<meter>(1.0),
            Length::new::<meter>(1.0),
            Length::new::<meter>(1.0),
        ];

        (altitudes, Duration::from_millis(500))
    }

    #[rstest]
    #[timeout(Duration::from_secs(10).into())]
    #[test_log::test(async_std::test)]
    async fn test_fsm(
        arm_button_signal: &'static Signal<CriticalSectionRawMutex, ()>,
        altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
        altitudes: (std::vec::Vec<Length>, Duration),
    ) {
        // --- Task: Simulate Flight States ---
        let arm_task = async move {
            let time_driver = MockDriver::get();

            time_driver.advance(Duration::from_secs(1));
            arm_button_signal.signal(());
            info!("Arming System!");

            async_std::task::sleep(Duration::from_millis(100).into()).await;

            let (altitudes, interval) = altitudes;
            for altitude in altitudes {
                altitude_signal.signal(altitude);
                time_driver.advance(interval);
                info!("Altitude: {} m", altitude.get::<meter>());
                async_std::task::sleep(Duration::from_millis(100).into()).await;
            }
        };

        // --- FSM Task ---
        let fsm_task = async move {
            let fsm = FiniteStateMachine::new(arm_button_signal);
            let fsm = fsm.wait_arm(altitude_signal).await;
            let fsm = fsm.wait_activate_recovery().await;
            let _ = fsm.wait_touchdown().await;
        };

        // --- Run all tasks concurrently ---
        join!(arm_task, fsm_task);
    }
}
