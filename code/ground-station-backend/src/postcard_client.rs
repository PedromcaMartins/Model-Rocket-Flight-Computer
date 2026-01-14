use std::sync::atomic::{AtomicU32, Ordering};

use postcard_rpc::{Endpoint, Topic, header::{VarSeq, VarSeqKind}, host_client::{HostClient, HostErr, IoClosed, SubscribeError, Subscription}, standard_icd::{ERROR_PATH, WireError}};
use proto::{GlobalTickHzEndpoint, PingEndpoint, PingRequest, PingResponse, Record, RecordTopic, Schema, SimAltimeterTopic, SimArmTopic, SimDeploymentTopic, SimGpsTopic, SimImuTopic, actuator_data::{ActuatorStatus, LedStatus}, record::tick_hz::GlobalTickHz, sensor_data::{AltimeterData, GpsData, ImuData}};
use serde::{Serialize, de::DeserializeOwned};
use tracing::{debug, trace};


#[derive(Debug, derive_more::From)]
pub enum PostcardError {
    #[from(HostErr<WireError>)]
    Comms(HostErr<WireError>),
    #[from(SubscribeError)]
    SubscriptionClosed,
    #[from(IoClosed)]
    IOClosed,
}

impl From<PostcardError> for String {
    fn from(err: PostcardError) -> Self {
        match err {
            PostcardError::Comms(e) => format!("Communication error: {e:?}"),
            PostcardError::SubscriptionClosed => "Subscription closed".to_string(),
            PostcardError::IOClosed => "IO closed".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct PostcardClient {
    client: HostClient<WireError>,
    seq_num: AtomicU32,
}

impl From<HostClient<WireError>> for PostcardClient {
    fn from(client: HostClient<WireError>) -> Self {
        Self { 
            client,
            seq_num: AtomicU32::new(0),
        }
    }
}

impl PostcardClient {
    pub fn try_new_raw_nusb() -> Result<Self, String> {
        let client = HostClient::try_new_raw_nusb(
            |d| d.product_string() == Some("flight_computer"),
            ERROR_PATH,
            8,
            VarSeqKind::Seq4,
        )?;

        Ok(Self { 
            client,
            seq_num: AtomicU32::new(0),
        })
    }

    pub async fn wait_closed(&self) {
        debug!("Waiting for postcard client to close...");
        self.client.wait_closed().await;
    }

    async fn service<E: Endpoint>(&self, req: &E::Request) -> Result<E::Response, PostcardError>
    where 
        E::Request: Serialize + Schema,
        E::Response: DeserializeOwned + Schema
    {
        trace!("Sending request to endpoint {}: {req:?}", E::NAME);
        self.client.send_resp::<E>(req).await.map_err(PostcardError::from)
    }

    pub async fn ping(&self, id: &PingRequest) -> Result<PingResponse, PostcardError> {
        self.service::<PingEndpoint>(id).await
    }

    pub async fn global_tick(&self) -> Result<GlobalTickHz, PostcardError> {
        self.service::<GlobalTickHzEndpoint>(&()).await
    }

    async fn subscribe<T: Topic>(&self) -> Result<Subscription<T::Message>, PostcardError>
    where
        T::Message: DeserializeOwned,
    {
        trace!("Subscribing to topic {}", T::NAME);
        self.client.subscribe_exclusive::<T>(u16::MAX.into()).await.map_err(PostcardError::from)
    }

    pub async fn subscribe_record(&self) -> Result<Subscription<Record>, PostcardError> {
        self.subscribe::<RecordTopic>().await
    }

    pub async fn subscribe_sim_deployment(&self) -> Result<Subscription<ActuatorStatus>, PostcardError> {
        self.subscribe::<SimDeploymentTopic>().await
    }

    pub async fn subscribe_led<T: Topic<Message = LedStatus>>(&self) -> Result<Subscription<LedStatus>, PostcardError> {
        self.subscribe::<T>().await
    }

    pub async fn publish<T: Topic>(&self, data: &T::Message) -> Result<(), PostcardError> 
    where
        T::Message: Serialize
    {
        trace!("Publishing to topic {}: {data:?}", T::NAME);
        self.client.publish::<T>(
            VarSeq::Seq4(self.seq_num.fetch_add(1, Ordering::Relaxed)),
            data,
        ).await.map_err(PostcardError::from)
    }

    pub async fn publish_sim_altimeter(&self, data: &AltimeterData) -> Result<(), PostcardError> {
        self.publish::<SimAltimeterTopic>(data).await
    }

    pub async fn publish_sim_gps(&self, data: &GpsData) -> Result<(), PostcardError> {
        self.publish::<SimGpsTopic>(data).await
    }

    pub async fn publish_sim_imu(&self, data: &ImuData) -> Result<(), PostcardError> {
        self.publish::<SimImuTopic>(data).await
    }

    pub async fn publish_sim_arm(&self, data: &ActuatorStatus) -> Result<(), PostcardError> {
        self.publish::<SimArmTopic>(data).await
    }
}
