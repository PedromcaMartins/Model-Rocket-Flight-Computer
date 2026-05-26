extern crate std;

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use postcard_rpc::{
    Endpoint, Topic,
    header::VarSeq,
    host_client::{HostClient, HostErr, IoClosed, SubscribeError, Subscription},
    standard_icd::WireError,
};
use serde::{Serialize, de::DeserializeOwned};

use crate::{DEFAULT_SUBSCRIBE_DEPTH, Schema};

/// Errors from [`PostcardClient`] operations.
#[derive(Debug, thiserror::Error)]
pub enum PostcardError {
    #[error("Communication error: {0:?}")]
    Comms(HostErr<WireError>),
    #[error("Subscription closed")]
    SubscriptionClosed(#[from] SubscribeError),
    #[error("IO closed")]
    IOClosed(#[from] IoClosed),
}

impl From<HostErr<WireError>> for PostcardError {
    fn from(e: HostErr<WireError>) -> Self {
        Self::Comms(e)
    }
}

/// A generic postcard-rpc client wrapper.
///
/// Wraps a [`HostClient<WireError>`] with a monotonic sequence number for
/// publish operations and convenience methods for sending endpoint requests
/// and subscribing to topics.
///
/// This is transport-agnostic — construct it from any [`HostClient`] returned
/// by a transport module (e.g. [`crate::transport::ipc::connect_client`] or
/// [`crate::transport::thread::create_pair`]).
///
/// Cloning shares the same sequence counter, so all clones maintain
/// monotonically increasing sequence numbers across publish calls.
#[derive(Clone)]
pub struct PostcardClient {
    client: HostClient<WireError>,
    seq: Arc<AtomicU32>,
}

impl From<HostClient<WireError>> for PostcardClient {
    fn from(client: HostClient<WireError>) -> Self {
        Self {
            client,
            seq: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl PostcardClient {
    /// Wrap an existing [`HostClient`].
    #[must_use]
    pub fn new(client: HostClient<WireError>) -> Self {
        client.into()
    }

    /// Wait for the underlying connection to close.
    pub async fn wait_closed(&self) {
        self.client.wait_closed().await;
    }

    /// Send an endpoint request and await the response.
    ///
    /// # Errors
    ///
    /// Returns [`PostcardError`] if the endpoint request fails or the
    /// connection is closed.
    pub async fn service<E: Endpoint>(&self, req: &E::Request) -> Result<E::Response, PostcardError>
    where
        E::Request: Serialize + Schema + Sync,
        E::Response: DeserializeOwned + Schema,
    {
        self.client
            .send_resp::<E>(req)
            .await
            .map_err(PostcardError::from)
    }

    /// Subscribe to a topic, returning a [`Subscription`] receiver.
    ///
    /// # Errors
    ///
    /// Returns [`PostcardError`] if the subscription fails or the
    /// connection is closed.
    pub async fn subscribe<T: Topic>(&self) -> Result<Subscription<T::Message>, PostcardError>
    where
        T::Message: DeserializeOwned,
    {
        self.client
            .subscribe_exclusive::<T>(DEFAULT_SUBSCRIBE_DEPTH.into())
            .await
            .map_err(PostcardError::from)
    }

    /// Publish a message on a topic with an auto-incrementing sequence number.
    ///
    /// # Errors
    ///
    /// Returns [`PostcardError`] if the publish fails or the connection is
    /// closed.
    pub async fn publish<T: Topic>(&self, msg: &T::Message) -> Result<(), PostcardError>
    where
        T::Message: Serialize + Sync,
    {
        self.client
            .publish::<T>(
                VarSeq::Seq4(self.seq.fetch_add(1, Ordering::Relaxed)),
                msg,
            )
            .await
            .map_err(PostcardError::from)
    }
}
