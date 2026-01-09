use proto::uom::si::length::meter;
use defmt_or_log::info;

use crate::{core::state_machine::{FlightStateMachine, detectors::TouchdownDetector, states::{RecoveryActivated, Touchdown}}, interfaces::{ArmingSystem, DeploymentSystem, Led}};

impl<A, LedA, D, LedD> FlightStateMachine<A, LedA, D, LedD, RecoveryActivated>
where
    A: ArmingSystem,
    LedA: Led,
    D: DeploymentSystem,
    LedD: Led,
{
    pub async fn wait_touchdown(self) -> FlightStateMachine<A, LedA, D, LedD, Touchdown> {
        let altitude = TouchdownDetector::new()
        .await
        .await_touchdown()
        .await;

        info!("Touchdown of {} m!", altitude.get::<meter>());

        self.transition()
    }
}
