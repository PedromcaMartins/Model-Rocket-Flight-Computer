use defmt_or_log::info;
use proto::flight_state::FlightState;

use crate::{core::state_machine::FlightStateMachine, interfaces::{ArmingSystem, DeploymentSystem, Led}, sync::broadcast_record};

#[inline]
pub async fn finite_state_machine_task<A, LedA, D, LedD>(
    arm_button: A, 
    arm_button_led: LedA,
    deployment_system: D, 
    deployment_system_led: LedD,
)
where
    A: ArmingSystem,
    LedA: Led,
    D: DeploymentSystem,
    LedD: Led,
{
    let fsm = FlightStateMachine::new(
        arm_button, 
        arm_button_led, 
        deployment_system, 
        deployment_system_led, 
    ).await;
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
