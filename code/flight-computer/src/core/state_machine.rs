use core::marker::PhantomData;

use proto::sensor_data::Altitude;

use crate::{core::state_machine::states::FlightState, interfaces::{ArmingSystem, DeploymentSystem, Led}};

mod states;
mod detectors;

pub struct FlightStateMachine<A, LedA, D, LedD, S>
where
    A: ArmingSystem,
    LedA: Led,
    D: DeploymentSystem,
    LedD: Led,
    S: FlightState,
{
    arm_button: A,
    arm_button_led: LedA,
    deployment_system: D,
    deployment_system_led: LedD,
    phantom_data: PhantomData<S>,

    /// The following attributes describe 
    launchpad_altitude: Option<Altitude>,
}

impl<A, LedA, D, LedD, S> FlightStateMachine<A, LedA, D, LedD, S>
where
    A: ArmingSystem,
    LedA: Led,
    D: DeploymentSystem,
    LedD: Led,
    S: FlightState,
{
    // Common transition helper
    fn transition<T: FlightState>(self) -> FlightStateMachine<A, LedA, D, LedD, T> {
        FlightStateMachine {
            arm_button: self.arm_button,
            arm_button_led: self.arm_button_led,
            deployment_system: self.deployment_system,
            deployment_system_led: self.deployment_system_led,
            launchpad_altitude: self.launchpad_altitude,
            phantom_data: PhantomData,
        }
    }
}
