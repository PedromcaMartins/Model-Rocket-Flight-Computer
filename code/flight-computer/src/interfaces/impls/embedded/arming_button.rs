use switch_hal::WaitSwitch;

use crate::interfaces::ArmingSystem;

pub struct ArmingButton<WS>
where
    WS: WaitSwitch,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
{
    button: WS,
}

impl<WS> ArmingButton<WS>
where
    WS: WaitSwitch,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
{
    pub const fn new(button: WS) -> Self {
        Self { button }
    }
}

impl<WS> ArmingSystem for ArmingButton<WS>
where
    WS: WaitSwitch,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
{
    type Error = WS::Error;

    async fn wait_arm(&mut self) -> Result<(), Self::Error> {
        self.button.wait_active().await
    }
}
