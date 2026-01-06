use switch_hal::WaitSwitch;
use proto::uom::si::length::meter;
use defmt_or_log::info;

use crate::{core::state_machine::{detectors::{TouchdownDetector}, states::{RecoveryActivated, Touchdown}, FlightStateMachine}, interfaces::DeploymentSystem};

impl<WS, D> FlightStateMachine<WS, D, RecoveryActivated>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
{
    pub async fn wait_touchdown(self) -> FlightStateMachine<WS, D, Touchdown> {
        let altitude = TouchdownDetector::new()
        .await
        .await_touchdown()
        .await;

        info!("Touchdown of {} m!", altitude.get::<meter>());

        self.transition()
    }
}
