use core::convert::Infallible;
use core::num::Wrapping;

use defmt_or_log::warn;
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use proto::{SimFileSystemLedTopic, actuator_data::LedStatus};

use crate::interfaces::Led;

pub struct SimFileSystemLed<'a, Tx: WireTx> {
    tx: &'a PostcardSender<Tx>,
    seq: Wrapping<u32>,
}

impl<'a, Tx: WireTx> SimFileSystemLed<'a, Tx> {
    pub fn new(tx: &'a PostcardSender<Tx>) -> Self {
        Self { 
            tx,
            seq: Wrapping::default(),
        }
    }
}

impl<Tx: WireTx> Led for SimFileSystemLed<'_, Tx> {
    type Error = Infallible;

    async fn off(&mut self) -> Result<(), Self::Error> {
        if self.tx.publish::<SimFileSystemLedTopic>(VarSeq::Seq4(self.seq.0), &LedStatus::Off).await.is_ok() {
            self.seq += 1;
        } else {
            warn!("SimRecovery: Failed to send deploy signal to simulator");
        }
        Ok(())
    }

    async fn on(&mut self) -> Result<(), Self::Error> {
        if self.tx.publish::<SimFileSystemLedTopic>(VarSeq::Seq4(self.seq.0), &LedStatus::On).await.is_ok() {
            self.seq += 1;
        } else {
            warn!("SimRecovery: Failed to send deploy signal to simulator");
        }
        Ok(())
    }
}
