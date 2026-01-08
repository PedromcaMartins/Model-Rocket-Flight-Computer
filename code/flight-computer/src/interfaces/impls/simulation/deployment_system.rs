use core::convert::Infallible;
use core::num::Wrapping;

use defmt_or_log::warn;
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use proto::{SimDeploymentTopic, actuator_data::ActuatorStatus};

use crate::interfaces::DeploymentSystem;

pub struct SimRecovery<'a, Tx: WireTx> {
    tx: &'a PostcardSender<Tx>,
    seq: Wrapping<u32>,
}

impl<'a, Tx: WireTx> SimRecovery<'a, Tx> {
    pub fn new(tx: &'a PostcardSender<Tx>) -> Self {
        Self { 
            tx,
            seq: Wrapping::default(),
        }
    }
}

impl<Tx: WireTx> DeploymentSystem for SimRecovery<'_, Tx> {
    type Error = Infallible;

    /// deploy recovery signal to simulator
    async fn deploy(&mut self) -> Result<(), Self::Error> {
        if self.tx.publish::<SimDeploymentTopic>(VarSeq::Seq4(self.seq.0), &ActuatorStatus::Active).await.is_ok() {
            self.seq += 1;
        } else {
            warn!("SimRecovery: Failed to send deploy signal to simulator");
        }
        Ok(())
    }
}
