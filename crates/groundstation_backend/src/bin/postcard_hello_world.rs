use std::time::Duration;

use telemetry_messages::{AltimeterTopic, GpsTopic, ImuTopic};
use tokio::time::interval;
use groundstation_backend::postcard_client::PostcardClient;

#[tokio::main]
pub async fn main() {
    let client = PostcardClient::new().await;

    let mut subscription_altimeter = client.client.subscribe_exclusive::<AltimeterTopic>(100).await
        .expect("Failed to subscribe to altimeter topic");

    let mut subscription_imu = client.client.subscribe_exclusive::<ImuTopic>(100).await
        .expect("Failed to subscribe to altimeter topic");

    let mut subscription_gps = client.client.subscribe_exclusive::<GpsTopic>(100).await
        .expect("Failed to subscribe to altimeter topic");

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
