use core::num::Wrapping;

use postcard_rpc::server::{AsWireTxErrorKind, Sender as PostcardSender, WireTx, WireTxErrorKind};
use postcard_rpc::header::VarSeq;
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
    type Error = WireTxErrorKind;

    async fn deploy(&mut self) -> Result<(), Self::Error> {
        self.tx.publish::<SimDeploymentTopic>(VarSeq::Seq4(self.seq.0), &ActuatorStatus::Active)
            .await
            .map_err(|e| e.as_kind())?;
        self.seq += 1;
        Ok(())
    }

    /// No simulator acknowledgment mechanism exists yet; treat a successful publish as confirmed.
    async fn verify_deployment(&mut self) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
