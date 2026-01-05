use std::convert::Infallible;

use switch_hal::WaitSwitch;
use tokio::sync::watch;

pub struct SimButton {
    rx: watch::Receiver<bool>,
}

impl SimButton {
    pub fn new(rx: watch::Receiver<bool>) -> Self {
        Self { rx }
    }
}

impl WaitSwitch for SimButton {
    type Error = Infallible;

    /// Wait for button press signal from simulator
    async fn wait_active(&mut self) -> Result<(), Self::Error> {
        loop {
            self.rx.changed().await.expect("Failed to wait for button state change: sender dropped");
            if *self.rx.borrow_and_update() {
                return Ok(());
            }
        }
    }
}
