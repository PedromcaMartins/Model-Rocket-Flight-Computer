use switch_hal::OutputSwitch;

use crate::interfaces::DeploymentSystem;

pub struct DeploymentSwitch<O>
where
    O: OutputSwitch,
    <O as OutputSwitch>::Error: core::fmt::Debug,
{
    switch: O,
}

impl<O> DeploymentSwitch<O>
where
    O: OutputSwitch,
    <O as OutputSwitch>::Error: core::fmt::Debug,
{
    pub const fn new(switch: O) -> Self {
        Self { switch }
    }
}

impl<O> DeploymentSystem for DeploymentSwitch<O>
where
    O: OutputSwitch,
    <O as OutputSwitch>::Error: core::fmt::Debug,
{
    type Error = O::Error;

    async fn deploy(&mut self) -> Result<(), Self::Error> {
        self.switch.on()
    }
}
