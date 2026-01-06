use std::convert::Infallible;

use crate::interfaces::DeploymentSystem;

pub struct SimParachute<Tx: WireTx> {
    tx: &PostcardSender<Tx>,
}

impl SimParachute {
    pub fn new(tx: &PostcardSender<Tx>) -> Self {
        Self { tx }
    }
}

impl DeploymentSystem for SimParachute {
    type Error = Infallible;

    /// deploy parachute signal to simulator
    fn deploy(&mut self) -> Result<(), Self::Error> {
        todo!("Requires postcard endpoint!");
        Ok(())
    }
}
