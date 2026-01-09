use defmt_or_log::info;
use proto::flight_state::FlightState;

use crate::{core::state_machine::FlightStateMachine, interfaces::{ArmingSystem, DeploymentSystem}, sync::broadcast_record};

#[inline]
pub async fn finite_state_machine_task<A, D>(arm_button: A, deployment_system: D)
where
    A: ArmingSystem,
    D: DeploymentSystem,
{
    let fsm = FlightStateMachine::new(
        arm_button, 
        deployment_system, 
    );
    update_flight_state(FlightState::default());

    let fsm = fsm.wait_arm().await;
    update_flight_state(FlightState::Armed);

    let fsm = fsm.wait_activate_recovery().await;
    update_flight_state(FlightState::RecoveryActivated);

    let fsm = fsm.wait_touchdown().await;
    update_flight_state(FlightState::Touchdown);

    fsm.shutdown().await;
}

fn update_flight_state(state: FlightState) {
    broadcast_record(state.into());
    info!("Flight Computer {state}");
}
