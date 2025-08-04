use std::time::Duration;

use postcard_rpc::host_client::Subscription;
use rstest::{fixture, rstest};
use telemetry_messages::ImuMessage;
use tokio::time::interval;
use ground_station_backend::PostcardClient;

#[fixture]
#[once]
fn client() -> PostcardClient {
    PostcardClient::new()
}

#[fixture]
async fn subscription_imu(client: &PostcardClient) -> Subscription<ImuMessage> {
    client.subscription_imu().await
        .expect("Failed to subscribe to altimeter topic")
}

#[rstest]
#[test_log::test(tokio::test)]
pub async fn subscription_imu_test(
    client: &PostcardClient,
    #[future] subscription_imu: Subscription<ImuMessage>,
) {
    let mut subscription_imu = subscription_imu.await;
    let mut ticker = interval(Duration::from_millis(250));

    for _ in 0..10 {
        ticker.tick().await;
        dbg!(subscription_imu.recv().await).unwrap();
    }
    
    client.client.close();
}
