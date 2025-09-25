use embassy_sync::blocking_mutex::raw::RawMutex;
use switch_hal::WaitSwitch;
use telemetry_messages::uom::si::length::meter;
use defmt_or_log::info;

use crate::{core::state_machine::{detectors::{TouchdownDetector}, states::{RecoveryActivated, Touchdown}, FlightStateMachine}, interfaces::DeploymentSystem};

impl<WS, D, M> FlightStateMachine<WS, D, M, RecoveryActivated>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static
{
    pub async fn wait_touchdown(self) -> FlightStateMachine<WS, D, M, Touchdown> {
        let altitude = TouchdownDetector::new(
            self.latest_altitude_signal,
            self.touchdown_detector_config,
        ).await
        .await_touchdown()
        .await;

        info!("Touchdown of {} m!", altitude.get::<meter>());

        self.transition_to(None)
    }
}
