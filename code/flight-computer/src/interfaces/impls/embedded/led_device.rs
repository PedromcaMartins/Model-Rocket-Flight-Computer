use switch_hal::{OutputSwitch, StatefulOutputSwitch};

use crate::interfaces::Led;

pub struct LedDevice<O>
where
    O: StatefulOutputSwitch,
    <O as OutputSwitch>::Error: core::fmt::Debug,
{
    led: O,
}

impl<O> LedDevice<O>
where
    O: StatefulOutputSwitch,
    <O as OutputSwitch>::Error: core::fmt::Debug,
{
    pub const fn new(led: O) -> Self {
        Self { led }
    }
}

impl<O> Led for LedDevice<O>
where
    O: StatefulOutputSwitch,
    <O as OutputSwitch>::Error: core::fmt::Debug,
{
    type Error = O::Error;

    async fn on(&mut self) -> Result<(), Self::Error> {
        self.led.on()
    }

    async fn off(&mut self) -> Result<(), Self::Error> {
        self.led.off()
    }

    async fn toggle(&mut self) -> Result<(), Self::Error> {
        self.led.toggle()
    }
}
