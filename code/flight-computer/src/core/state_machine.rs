use core::marker::PhantomData;

use proto::sensor_data::Altitude;

use crate::{core::state_machine::states::FlightState, interfaces::{ArmingSystem, DeploymentSystem}};

mod states;
mod detectors;

pub struct FlightStateMachine<A, D, S>
where
    A: ArmingSystem,
    D: DeploymentSystem,
    S: FlightState,
{
    arm_button: A,
    deployment_system: D,
    phantom_data: PhantomData<S>,

    /// The following attributes describe 
    launchpad_altitude: Option<Altitude>,
}

impl<A, D, S> FlightStateMachine<A, D, S>
where
    A: ArmingSystem,
    D: DeploymentSystem,
    S: FlightState,
{
    // Common transition helper
    fn transition<T: FlightState>(self) -> FlightStateMachine<A, D, T> {
        FlightStateMachine {
            arm_button: self.arm_button,
            deployment_system: self.deployment_system,
            launchpad_altitude: self.launchpad_altitude,
            phantom_data: PhantomData,
        }
    }
}
