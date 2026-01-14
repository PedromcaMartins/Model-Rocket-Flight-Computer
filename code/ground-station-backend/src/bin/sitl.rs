use ground_station_backend::{ApiConfig, PostcardClient, start_api};

#[tokio::main]
pub async fn main() {
    let (server, client) = postcard_local_setup();

    let config = ApiConfig::default();

    start_api(client, config).await.expect("Failed to start API server");
    // start_simulator(server).await;
}
