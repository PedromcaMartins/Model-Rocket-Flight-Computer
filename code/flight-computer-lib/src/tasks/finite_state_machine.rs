use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use uom::si::f64::Length;

use crate::model::finite_state_machine::FiniteStateMachine;

#[derive(Debug, Clone)]
pub enum FiniteStateMachineStatus {
    PreArmed,
    Armed,
    RecoveryActivated,
    Touchdown,
}

#[inline]
pub async fn finite_state_machine_task<M>(
    arm_button_signal: &'static Signal<M, ()>,
    altitude_signal: &'static Signal<M, Length>,
)
where
    M: RawMutex + 'static,
{
    let fsm = FiniteStateMachine::new(arm_button_signal);
    let mut status = FiniteStateMachineStatus::PreArmed;

    let fsm = fsm.wait_arm(altitude_signal).await;
    status = FiniteStateMachineStatus::Armed;

    let fsm = fsm.wait_activate_recovery().await;
    status = FiniteStateMachineStatus::RecoveryActivated;

    let _ = fsm.wait_touchdown().await;
    status = FiniteStateMachineStatus::Touchdown;
}
