use defmt_or_log::info;

use crate::{core::state_machine::{FlightStateMachine, states::Touchdown}, interfaces::{ArmingSystem, DeploymentSystem, Led}};

impl<A, LedA, D, LedD> FlightStateMachine<A, LedA, D, LedD, Touchdown>
where
    A: ArmingSystem,
    LedA: Led,
    D: DeploymentSystem,
    LedD: Led,
{
    pub async fn shutdown(self) {
        info!("Shutting down flight computer.");
    }
}
