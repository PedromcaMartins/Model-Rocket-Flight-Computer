use std::{sync::Arc, time::Duration};
use rocket::Shutdown;
#[allow(unused_imports)]
use rocket::{
    serde::{json::{serde_json, Json}, Deserialize, Serialize},
    State, response::stream::{Event, EventStream}, tokio::select,
};
use proto::{PingRequest, PingResponse};
use tokio::{sync::Mutex, time::sleep};

use crate::{postcard_client::PostcardClient};

type SharedClient = Arc<Mutex<PostcardClient>>;

pub struct ApiConfig {
    pub base_path: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_path: "/api".to_string(),
        }
    }
}

pub async fn start_api(client: PostcardClient, config: ApiConfig) -> Result<(), rocket::Error> {
    let client = Arc::new(Mutex::new(client));

    let _rocket = rocket::build()
        .manage(client)
        .mount(config.base_path, routes())
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}

fn routes() -> Vec<rocket::Route> {
    rocket::routes![
    ]
}
