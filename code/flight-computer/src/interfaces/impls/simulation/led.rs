use core::convert::Infallible;
use core::num::Wrapping;

use defmt_or_log::error;
use postcard_rpc::{Topic, header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use proto::actuator_data::LedStatus;

use crate::interfaces::Led;

pub struct SimLed<'a, Tx, T>
where
    Tx: WireTx,
    T: Topic<Message = LedStatus>,
{
    tx: &'a PostcardSender<Tx>,
    seq: Wrapping<u32>,
    _topic: core::marker::PhantomData<T>,
}

impl<'a, Tx, T> SimLed<'a, Tx, T>
where
    Tx: WireTx,
    T: Topic<Message = LedStatus>,
{
    pub fn new(tx: &'a PostcardSender<Tx>) -> Self {
        Self {
            tx,
            seq: Wrapping::default(),
            _topic: core::marker::PhantomData,
        }
    }
}

impl<Tx, T> Led for SimLed<'_, Tx, T>
where
    Tx: WireTx,
    T: Topic<Message = LedStatus>,
{
    type Error = Infallible;

    async fn off(&mut self) -> Result<(), Self::Error> {
        if self
            .tx
            .publish::<T>(VarSeq::Seq4(self.seq.0), &LedStatus::Off)
            .await
            .is_ok()
        {
            self.seq += 1;
        } else {
            error!("SimLed: Failed to send led status (Off)");
        }
        Ok(())
    }

    async fn on(&mut self) -> Result<(), Self::Error> {
        if self
            .tx
            .publish::<T>(VarSeq::Seq4(self.seq.0), &LedStatus::On)
            .await
            .is_ok()
        {
            self.seq += 1;
        } else {
            error!("SimLed: Failed to send led status (On)");
        }
        Ok(())
    }
}
