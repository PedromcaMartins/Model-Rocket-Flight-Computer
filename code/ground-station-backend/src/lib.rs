use std::convert::Infallible;

use postcard_rpc::{header::VarSeqKind, host_client::{HostClient, HostErr}, standard_icd::{WireError, ERROR_PATH}};
use telemetry_messages::PingEndpoint;


pub struct PostcardClient {
    pub client: HostClient<WireError>,
}

#[derive(Debug)]
pub enum PostcardError<E> {
    Comms(HostErr<WireError>),
    Endpoint(E),
    SubscriptionClosed,
}

impl<E> From<HostErr<WireError>> for PostcardError<E> {
    fn from(value: HostErr<WireError>) -> Self {
        Self::Comms(value)
    }
}

// ---

impl PostcardClient {
    pub async fn new() -> Self {
        let client = HostClient::new_raw_nusb(
            |d| d.product_string() == Some("flight_computer"),
            ERROR_PATH,
            8,
            VarSeqKind::Seq2,
        );

        Self { 
            client,
        }
    }

    pub async fn wait_closed(&self) {
        self.client.wait_closed().await;
    }

    pub async fn ping(&self, id: u32) -> Result<u32, PostcardError<Infallible>> {
        let val = self.client.send_resp::<PingEndpoint>(&id).await?;
        Ok(val)
    }
}
