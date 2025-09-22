use std::convert::Infallible;

use flight_computer_lib::interfaces::DeploymentSystem;
use tokio::sync::watch;

pub struct SimParachute {
    tx: watch::Sender<bool>,
}

impl SimParachute {
    pub fn new(tx: watch::Sender<bool>) -> Self {
        Self { tx }
    }
}

impl DeploymentSystem for SimParachute {
    type Error = Infallible;

    /// deploy parachute signal to simulator
    fn deploy(&mut self) -> Result<(), Self::Error> {
        self.tx.send(true).expect("Failed to send parachute deployment signal: receiver dropped");
        Ok(())
    }
}
