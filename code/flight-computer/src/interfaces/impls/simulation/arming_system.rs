use core::convert::Infallible;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use crate::interfaces::ArmingSystem;

static LATEST_DATA: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub struct SimArming;
impl SimArming {
    pub async fn activate() {
        LATEST_DATA.signal(());
    }
}

impl ArmingSystem for SimArming {
    type Error = Infallible;

    /// Wait for button press signal from simulator
    async fn wait_arm(&mut self) -> Result<(), Self::Error> {
        LATEST_DATA.wait().await;
        Ok(())
    }
}
