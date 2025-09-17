use std::convert::Infallible;

use postcard_rpc::{header::VarSeqKind, host_client::{HostClient, HostErr, SubscribeError, Subscription}, standard_icd::{WireError, ERROR_PATH}};
use telemetry_messages::{AltimeterMessage, AltimeterTopic, GpsMessage, GpsTopic, ImuMessage, ImuTopic, PingEndpoint};


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

// ---

impl PostcardClient {
    pub fn new(client: HostClient<WireError>) -> Self {
        Self { client }
    }

    pub fn new_raw_nusb() -> Self {
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
        self.client.send_resp::<PingEndpoint>(&id).await.map_err(PostcardError::from)
    }

    pub async fn subscription_altimeter(&self) -> Result<Subscription<AltimeterMessage>, PostcardError<Infallible>> {
        self.client.subscribe_exclusive::<AltimeterTopic>(usize::MAX).await.map_err(PostcardError::from)
    }

    pub async fn subscription_imu(&self) -> Result<Subscription<ImuMessage>, PostcardError<Infallible>> {
        self.client.subscribe_exclusive::<ImuTopic>(usize::MAX).await.map_err(PostcardError::from)
    }

    pub async fn subscription_gps(&self) -> Result<Subscription<GpsMessage>, PostcardError<Infallible>> {
        self.client.subscribe_exclusive::<GpsTopic>(usize::MAX).await.map_err(PostcardError::from)
    }
}
