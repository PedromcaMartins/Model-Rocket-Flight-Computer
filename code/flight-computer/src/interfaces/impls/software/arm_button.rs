use std::convert::Infallible;

use switch_hal::WaitSwitch;

pub struct SimButton;
impl SimButton {
    const LATEST_DATA: Signal<CriticalSectionRawMutex, ()> = Signal::new();

    pub async fn activate_button() {
        Self::LATEST_DATA.signal(());
    }
}

impl WaitSwitch for SimButton {
    type Error = Infallible;

    /// Wait for button press signal from simulator
    async fn wait_active(&mut self) -> Result<(), Self::Error> {
        Ok(Self::LATEST_DATA.wait().await)
    }
}
