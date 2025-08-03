use std::time::Duration;

use postcard_rpc::host_client::Subscription;
use rstest::{fixture, rstest};
use telemetry_messages::{AltimeterMessage, AltimeterTopic, GpsMessage, GpsTopic, ImuMessage, ImuTopic};
use tokio::time::interval;
use ground_station_backend::PostcardClient;

#[fixture]
async fn client() -> (
    PostcardClient, 
    Subscription<AltimeterMessage>, 
    Subscription<ImuMessage>, 
    Subscription<GpsMessage>
) {
    let client = PostcardClient::new().await;

    let subscription_altimeter = client.client.subscribe_exclusive::<AltimeterTopic>(100).await
        .expect("Failed to subscribe to altimeter topic");

    let subscription_imu = client.client.subscribe_exclusive::<ImuTopic>(100).await
        .expect("Failed to subscribe to altimeter topic");

    let subscription_gps = client.client.subscribe_exclusive::<GpsTopic>(100).await
        .expect("Failed to subscribe to altimeter topic");

    (client, subscription_altimeter, subscription_imu, subscription_gps)
}

#[rstest]
#[test_log::test(tokio::test)]
pub async fn logging(
    #[future] client: (
        PostcardClient,
        Subscription<AltimeterMessage>,
        Subscription<ImuMessage>,
        Subscription<GpsMessage>,
    )
) {
    let (
        client,
        mut subscription_altimeter,
        mut subscription_imu,
        mut subscription_gps,
    ) = client.await;

    tokio::select! {
        _ = client.wait_closed() => {
            println!("Client is closed, exiting...");
        }
        _ = async {
            let mut ticker = interval(Duration::from_millis(250));
        
            for i in 0..10 {
                ticker.tick().await;
                print!("Pinging with {i}... ");
                let res = client.ping(i).await.unwrap();
                println!("got {res}!");
                assert_eq!(res, i);
            }
        } => {}
        _ = async move {
            let mut ticker = interval(Duration::from_millis(250));

            for _ in 0..10 {
                ticker.tick().await;
                let msg = subscription_altimeter.recv().await;
                println!("Got altimeter message: {msg:#?}");
            }
        } => {}
        _ = async move {
            let mut ticker = interval(Duration::from_millis(250));

            for _ in 0..10 {
                ticker.tick().await;
                let msg = subscription_imu.recv().await;
                println!("Got imu message: {msg:#?}");
            }
        } => {}
        _ = async move {
            let mut ticker = interval(Duration::from_millis(250));

            for _ in 0..10 {
                ticker.tick().await;
                let msg = subscription_gps.recv().await;
                println!("Got gps message: {msg:#?}");
            }
        } => {}
    }
}
