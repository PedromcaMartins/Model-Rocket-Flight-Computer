use std::time::Duration;

use rstest::{fixture, rstest};
use tokio::time::interval;
use ground_station_backend::PostcardClient;

#[fixture]
#[once]
fn client() -> PostcardClient {
    PostcardClient::default()
}

#[rstest]
#[test_log::test(tokio::test)]
pub async fn ping(client: &PostcardClient) {
    let mut ticker = interval(Duration::from_millis(250));

    for i in 0..10 {
        ticker.tick().await;
        dbg!("Pinging with {i}... ");
        let res = client.ping(i).await.unwrap();
        dbg!("got {res}!");
        assert_eq!(res, i);
    }

    client.client.close();
}
