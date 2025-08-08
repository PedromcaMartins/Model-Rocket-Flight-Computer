use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::Timer;
use switch_hal::WaitSwitch;

#[derive(Debug, Clone, Default)]
pub struct SystemStatus {
    pub arm_button_pressed: u64,
    pub failed_to_read_arm_button: u64,
}

#[inline]
pub async fn arm_button_task<S, M>(
    mut arm_button: S,
    arm_button_signal: &'static Signal<M, ()>,
) -> !
where
    S: WaitSwitch + 'static,
    <S as WaitSwitch>::Error: core::fmt::Debug,
    M: RawMutex + 'static,
{
    let mut status = SystemStatus::default();

    loop {
        // Wait for the button to be pressed
        if arm_button.wait_active().await.is_err() {
            status.failed_to_read_arm_button += 1;
            Timer::after_millis(1_000).await;
            continue;
        }

        // Notify the finite state machine that the arm button was pressed
        arm_button_signal.signal(());
        status.arm_button_pressed += 1;
    }
}
