use ground_station_backend::{ApiConfig, GroundStationConfig, Logging, handle_flight_computer, start_ground_station};
use tracing::error;

#[tokio::main]
pub async fn main() {
    let config = GroundStationConfig::default();
    Logging::init(config.logging).await;

    let client = match PostcardClient::try_new_raw_nusb() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create PostcardClient: {e:?}");
            return;
        }
    };

    tokio::spawn(handle_flight_computer(client)).await.expect("Failed to start ground station");
}
