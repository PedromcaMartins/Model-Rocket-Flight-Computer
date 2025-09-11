use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::Timer;
use switch_hal::WaitSwitch;
use defmt_or_log::{error, info};

#[inline]
pub async fn arm_button_task<
    S, M,
> (
    mut arm_button: S,
    arm_button_pushed_signal: &'static Signal<M, ()>,
) -> !
where
    S: WaitSwitch + 'static,
    <S as WaitSwitch>::Error: core::fmt::Debug,
    M: RawMutex + 'static,
{
    loop {
        if arm_button.wait_active().await.is_ok() {
            info!("Arm button pressed");
            arm_button_pushed_signal.signal(());
        } else {
            error!("Arm button: Failed to wait for button press");
            Timer::after_secs(1).await;
        }
    }
}
