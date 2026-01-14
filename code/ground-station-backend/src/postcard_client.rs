use std::convert::Infallible;

use postcard_rpc::{header::VarSeqKind, host_client::{HostClient, HostErr, SubscribeError, Subscription}, standard_icd::{WireError, ERROR_PATH}};
use proto::{Record, RecordTopic, sensor_data::{AltimeterData, AltimeterTopic, GpsData, GpsTopic, ImuData, ImuTopic, PingEndpoint, PingRequest, PingResponse}};


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

impl From<SubscribeError> for PostcardError<Infallible> {
    fn from(_: SubscribeError) -> Self {
        Self::SubscriptionClosed
    }
}

impl From<HostClient<WireError>> for PostcardClient {
    fn from(client: HostClient<WireError>) -> Self {
        Self {
            client
        }
    }
}

impl PostcardClient {
    pub fn try_new_raw_nusb() -> Result<Self, String> {
        let client = HostClient::try_new_raw_nusb(
            |d| d.product_string() == Some("flight_computer"),
            ERROR_PATH,
            8,
            VarSeqKind::Seq2,
        )?;

        Ok(Self { 
            client,
        })
    }

    pub async fn wait_closed(&self) {
        self.client.wait_closed().await;
    }

    pub async fn ping(&self, id: PingRequest) -> Result<PingResponse, PostcardError<Infallible>> {
        self.client.send_resp::<PingEndpoint>(&id.into()).await.map_err(PostcardError::from)
    }

    pub async fn subscription_record(&self) -> Result<Subscription<Record>, PostcardError<Infallible>> {
        self.client.subscribe_exclusive::<RecordTopic>(u16::MAX.into()).await.map_err(PostcardError::from)
    }
}
