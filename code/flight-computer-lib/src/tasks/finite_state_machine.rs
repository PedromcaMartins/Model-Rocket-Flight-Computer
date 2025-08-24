use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal, watch::{self}};
use uom::si::f64::Length;

use crate::model::{finite_state_machine::FiniteStateMachine, system_status::FiniteStateMachineStatus};

#[inline]
pub async fn finite_state_machine_task<
    M, 
    const N_RECEIVERS: usize,
> (
    arm_button_signal: &'static Signal<M, ()>,
    altitude_signal: &'static Signal<M, Length>,
    status_signal: watch::Sender<'static, M, FiniteStateMachineStatus, N_RECEIVERS>,
)
where
    M: RawMutex + 'static,
{
    let fsm = FiniteStateMachine::new(arm_button_signal);
    status_signal.send(FiniteStateMachineStatus::PreArmed);

    let fsm = fsm.wait_arm(altitude_signal).await;
    status_signal.send(FiniteStateMachineStatus::Armed);

    let fsm = fsm.wait_activate_recovery().await;
    status_signal.send(FiniteStateMachineStatus::RecoveryActivated);

    let _ = fsm.wait_touchdown().await;
    status_signal.send(FiniteStateMachineStatus::Touchdown);
}
