use std::time::Duration;

use tokio::time::interval;
use groundstation_backend::postcard_client::PostcardClient;

#[tokio::main]
pub async fn main() {
    let client = PostcardClient::new();

    // let uid = client.get_id().await.unwrap();
    // println!("uid: {:#?}", uid);

    tokio::select! {
        _ = client.wait_closed() => {
            println!("Client is closed, exiting...");
        }
        _ = run(&client) => {
            println!("App is done")
        }
    }
}

async fn run(client: &PostcardClient) {
    let mut ticker = interval(Duration::from_millis(250));

    for i in 0..10 {
        ticker.tick().await;
        print!("Pinging with {i}... ");
        let res = client.ping(i).await.unwrap();
        println!("got {res}!");
        assert_eq!(res, i);
    }
}
