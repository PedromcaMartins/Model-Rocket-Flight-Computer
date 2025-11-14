use ground_station_backend::{ApiConfig, PostcardClient, start_api};

#[tokio::main]
pub async fn main() {
    let client = match PostcardClient::try_new_raw_nusb() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create PostcardClient: {e:?}");
            return;
        }
    };
    let config = ApiConfig::default();

    start_api(client, config).await.expect("Failed to start API server");
}
