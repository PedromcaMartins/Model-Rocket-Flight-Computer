use core::convert::Infallible;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use switch_hal::WaitSwitch;

static LATEST_DATA: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub struct SimButton;
impl SimButton {

    pub async fn activate_button() {
        LATEST_DATA.signal(());
    }
}

impl WaitSwitch for SimButton {
    type Error = Infallible;

    /// Wait for button press signal from simulator
    async fn wait_active(&mut self) -> Result<(), Self::Error> {
        LATEST_DATA.wait().await;
        Ok(())
    }
}
