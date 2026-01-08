use defmt_or_log::info;

use crate::{core::state_machine::{FlightStateMachine, states::Touchdown}, interfaces::{ArmingSystem, DeploymentSystem}};

impl<A, D> FlightStateMachine<A, D, Touchdown>
where
    A: ArmingSystem,
    D: DeploymentSystem,
{
    pub async fn shutdown(self) {
        info!("Shutting down flight computer.");
    }
}
