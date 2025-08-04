use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embedded_hal_async::digital::Wait;

#[inline]
pub async fn arm_button_task<M, P>(
    mut arm_button: P,
    arm_button_signal: &'static Signal<M, ()>,
) -> !
where
    M: RawMutex + 'static,
    P: Wait + 'static,
{
    loop {
        // Wait for the button to be pressed
        arm_button.wait_for_high().await.unwrap();

        // Notify the finite state machine that the arm button was pressed
        arm_button_signal.signal(());

        // Wait for the button to be released
        arm_button.wait_for_low().await.unwrap();
    }
}
