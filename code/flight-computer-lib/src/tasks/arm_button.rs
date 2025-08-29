use core::{num::Saturating};

use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Sender, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use switch_hal::WaitSwitch;

use crate::{error_sending_to_system_status, model::system_status::ArmButtonSystemStatus, send_to_system_status};

#[inline]
pub async fn arm_button_task<
    S, M,
    const DEPTH_STATUS: usize,
> (
    mut arm_button: S,
    arm_button_pushed_signal: &'static Signal<M, ()>,
    status_sender: Sender<'static, M, Result<ArmButtonSystemStatus, usize>, DEPTH_STATUS>,
) -> !
where
    S: WaitSwitch + 'static,
    <S as WaitSwitch>::Error: core::fmt::Debug,
    M: RawMutex + 'static,
{
    let mut error_sending_status = Saturating::default();
    let mut status_timeout = Instant::now();

    loop {
        match select (
            arm_button.wait_active(),
            Timer::at(status_timeout),
        ).await {
            Either::First(Ok(())) => {
                // Notify the finite state machine that the arm button was pressed
                send_to_system_status!(status_sender, error_sending_status, ArmButtonSystemStatus::ArmButtonPressed);
                arm_button_pushed_signal.signal(());
            },
            Either::First(Err(_)) => {
                send_to_system_status!(status_sender, error_sending_status, ArmButtonSystemStatus::FailedToReadArmButton);
                Timer::after_secs(1).await;
            },
            Either::Second(()) => {
                error_sending_to_system_status!(status_sender, error_sending_status);
                status_timeout = Instant::now() + Duration::from_secs(1);
            },
        }
    }
}
