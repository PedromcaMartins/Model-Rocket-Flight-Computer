use defmt_or_log::{info, error, Debug2Format};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use switch_hal::WaitSwitch;

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
    loop {
        // Wait for the button to be pressed
        match arm_button.wait_active().await {
            Ok(()) => {
                // Notify the finite state machine that the arm button was pressed
                arm_button_signal.signal(());
                info!("Arm button pressed");
            },
            // Err(e) => error!("Failed to read arm button: {:?}", Debug2Format(&e)),
            Err(_) => (),
        }
    }
}
