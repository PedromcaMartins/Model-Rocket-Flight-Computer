use std::{sync::Arc, time::Duration};
use rocket::Shutdown;
#[allow(unused_imports)]
use rocket::{
    serde::{json::{serde_json, Json}, Deserialize, Serialize},
    State, response::stream::{Event, EventStream}, tokio::select,
};
use telemetry_messages::{PingRequest, PingResponse};
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
        ping, 
        stream_altimeter, 
        stream_imu, 
        stream_gps
    ]
}

#[rocket::post("/ping", data = "<req>")]
async fn ping(
    state: &State<SharedClient>,
    req: Json<PingRequest>,
) -> Result<Json<PingResponse>, String> {
    let client = state.inner().clone();
    let id = req.0;
    match client.lock().await.ping(id.into()).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err(format!("Ping failed: {e:?}")),
    }
}

#[rocket::get("/stream/altimeter")]
async fn stream_altimeter(
    state: &State<SharedClient>, 
    mut end: Shutdown,
) -> EventStream![] {
    let client = state.inner().clone();

    EventStream! {
        let mut sub = match client.lock().await.subscription_altimeter().await {
            Ok(s) => s,
            Err(e) => {
                yield Event::data(format!("error: {e:?}"));
                return;
            }
        };

        loop {
            select! {
                msg = sub.recv() => {
                    if let Some(data) = msg {
                        let json = serde_json::to_string(&data).unwrap();
                        yield Event::data(json);
                    } else {
                        yield Event::data("subscription closed");
                        break;
                    }
                }
                _ = sleep(Duration::from_secs(1)) => { /* heartbeat */ }
                _ = &mut end => break
            }
        }
    }
}

#[rocket::get("/stream/imu")]
async fn stream_imu(
    state: &State<SharedClient>, 
    mut end: Shutdown
) -> EventStream![] {
    let client = state.inner().clone();

    EventStream! {
        let mut sub = match client.lock().await.subscription_imu().await {
            Ok(s) => s,
            Err(e) => {
                yield Event::data(format!("error: {e:?}"));
                return;
            }
        };

        loop {
            select! {
                msg = sub.recv() => {
                    if let Some(data) = msg {
                        let json = serde_json::to_string(&data).unwrap();
                        yield Event::data(json);
                    } else {
                        yield Event::data("subscription closed");
                        break;
                    }
                }
                _ = sleep(Duration::from_secs(1)) => {}
                _ = &mut end => break
            }
        }
    }
}

#[rocket::get("/stream/gps")]
async fn stream_gps(
    state: &State<SharedClient>, 
    mut end: Shutdown
) -> EventStream![] {
    let client = state.inner().clone();

    EventStream! {
        let mut sub = match client.lock().await.subscription_gps().await {
            Ok(s) => s,
            Err(e) => {
                yield Event::data(format!("error: {e:?}"));
                return;
            }
        };

        loop {
            select! {
                msg = sub.recv() => {
                    if let Some(data) = msg {
                        let json = serde_json::to_string(&data).unwrap();
                        yield Event::data(json);
                    } else {
                        yield Event::data("subscription closed");
                        break;
                    }
                }
                _ = sleep(Duration::from_secs(1)) => {}
                _ = &mut end => break
            }
        }
    }
}
