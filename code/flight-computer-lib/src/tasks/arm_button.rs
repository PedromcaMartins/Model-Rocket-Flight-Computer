use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use switch_hal::WaitSwitch;

use crate::model::system_status::ArmButtonSystemStatus;

#[inline]
pub async fn arm_button_task<S, M>(
    mut arm_button: S,
    arm_button_signal: &'static Signal<M, ()>,
    status_signal: &'static Signal<M, ArmButtonSystemStatus>,
) -> !
where
    S: WaitSwitch + 'static,
    <S as WaitSwitch>::Error: core::fmt::Debug,
    M: RawMutex + 'static,
{
    let mut status = ArmButtonSystemStatus::default();
    let mut status_timeout = Instant::now();

    loop {
        match select (
            arm_button.wait_active(),
            Timer::at(status_timeout),
        ).await {
            Either::First(Err(_)) => {
                // Wait for the button to be pressed
                status.failed_to_read_arm_button += 1;
                Timer::after_millis(1_000).await;
            },
            Either::First(Ok(())) => {
                // Notify the finite state machine that the arm button was pressed
                status.arm_button_pressed += 1;
                arm_button_signal.signal(());
            },
            Either::Second(()) => {
                status_signal.signal(status.clone());
                status_timeout = Instant::now() + Duration::from_millis(1_000);
            },
        }
    }
}
