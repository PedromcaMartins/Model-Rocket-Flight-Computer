use std::time::Duration;

use postcard_rpc::host_client::Subscription;
use rstest::{fixture, rstest};
use telemetry_messages::GpsMessage;
use tokio::time::interval;
use ground_station_backend::PostcardClient;

#[fixture]
#[once]
fn client() -> PostcardClient {
    PostcardClient::default()
}

#[fixture]
async fn subscription_gps(client: &PostcardClient) -> Subscription<GpsMessage> {
    client.subscription_gps().await
        .expect("Failed to subscribe to altimeter topic")
}

#[rstest]
#[test_log::test(tokio::test)]
pub async fn subscription_gps_test(
    client: &PostcardClient,
    #[future] subscription_gps: Subscription<GpsMessage>,
) {
    let mut subscription_gps = subscription_gps.await;
    let mut ticker = interval(Duration::from_millis(250));

    for _ in 0..10 {
        ticker.tick().await;
        dbg!(subscription_gps.recv().await).unwrap();
    }

    client.client.close();
}
