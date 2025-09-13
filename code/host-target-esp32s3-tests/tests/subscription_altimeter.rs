use std::time::Duration;

use postcard_rpc::host_client::Subscription;
use rstest::{fixture, rstest};
use telemetry_messages::AltimeterMessage;
use tokio::time::interval;
use ground_station_backend::PostcardClient;

#[fixture]
#[once]
fn client() -> PostcardClient {
    PostcardClient::default()
}

#[fixture]
async fn subscription_altimeter(client: &PostcardClient) -> Subscription<AltimeterMessage> {
    client.subscription_altimeter().await
        .expect("Failed to subscribe to altimeter topic")
}

#[rstest]
#[test_log::test(tokio::test)]
pub async fn subscription_altimeter_test(
    client: &PostcardClient,
    #[future] subscription_altimeter: Subscription<AltimeterMessage>,
) {
    let mut subscription_altimeter = subscription_altimeter.await;
    let mut ticker = interval(Duration::from_millis(250));

    for _ in 0..10 {
        ticker.tick().await;
        dbg!(subscription_altimeter.recv().await).unwrap();
    }

    client.client.close();
}
