use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use uom::si::f64::Length;

use crate::model::finite_state_machine::FiniteStateMachine;

#[inline]
pub async fn finite_state_machine_task<M>(
    arm_button_signal: &'static Signal<M, ()>,
    altitude_signal: &'static Signal<M, Length>,
)
where
    M: RawMutex + 'static,
{
    let fsm = FiniteStateMachine::new(arm_button_signal);
    let fsm = fsm.wait_arm(altitude_signal).await;
    let fsm = fsm.wait_activate_recovery().await;
    let _ = fsm.wait_touchdown().await;
}
