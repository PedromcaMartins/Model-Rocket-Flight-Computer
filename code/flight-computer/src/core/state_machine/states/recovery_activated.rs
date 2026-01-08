use proto::uom::si::length::meter;
use defmt_or_log::info;

use crate::{core::state_machine::{FlightStateMachine, detectors::TouchdownDetector, states::{RecoveryActivated, Touchdown}}, interfaces::{ArmingSystem, DeploymentSystem}};

impl<A, D> FlightStateMachine<A, D, RecoveryActivated>
where
    A: ArmingSystem,
    D: DeploymentSystem,
{
    pub async fn wait_touchdown(self) -> FlightStateMachine<A, D, Touchdown> {
        let altitude = TouchdownDetector::new()
        .await
        .await_touchdown()
        .await;

        info!("Touchdown of {} m!", altitude.get::<meter>());

        self.transition()
    }
}
