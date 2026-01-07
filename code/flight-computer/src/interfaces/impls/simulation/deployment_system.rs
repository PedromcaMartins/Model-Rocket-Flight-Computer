use core::convert::Infallible;

use postcard_rpc::server::{Sender as PostcardSender, WireTx};

use crate::interfaces::DeploymentSystem;

pub struct SimParachute<'a, Tx: WireTx> {
    tx: &'a PostcardSender<Tx>,
}

impl<'a, Tx: WireTx> SimParachute<'a, Tx> {
    pub const fn new(tx: &'a PostcardSender<Tx>) -> Self {
        Self { tx }
    }
}

impl<Tx: WireTx> DeploymentSystem for SimParachute<'_, Tx> {
    type Error = Infallible;

    /// deploy parachute signal to simulator
    fn deploy(&mut self) -> Result<(), Self::Error> {
        todo!("Requires postcard endpoint!");
        Ok(())
    }
}
