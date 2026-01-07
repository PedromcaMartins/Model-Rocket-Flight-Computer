use core::convert::Infallible;

use postcard_rpc::server::{Sender as PostcardSender, WireTx};
use switch_hal::OutputSwitch;

pub struct SimSdCardLed<'a, Tx: WireTx> {
    tx: &'a PostcardSender<Tx>,
}

impl<'a, Tx: WireTx> SimSdCardLed<'a, Tx> {
    pub const fn new(tx: &'a PostcardSender<Tx>) -> Self {
        Self { tx }
    }
}

impl<Tx: WireTx> OutputSwitch for SimSdCardLed<'_, Tx> {
    type Error = Infallible;

    fn off(&mut self) -> Result<(), Self::Error> {
        todo!("Requires postcard endpoint!");
        Ok(())
    }

    fn on(&mut self) -> Result<(), Self::Error> {
        todo!("Requires postcard endpoint!");
        Ok(())
    }
}
