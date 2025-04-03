use std::{path::PathBuf, str::FromStr};

use defmt_parser::Level;
use groundstation::{parser::{LocationMessage, LogMessage, ModulePath}, GroundStation};
use time::OffsetDateTime;
use tokio::{sync::mpsc, time::Instant};

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let groundstation = GroundStation::default();

    tokio::spawn(simulated_telem(groundstation.clone_tx()));

    // Spawn the GUI in a separate thread
    eframe::run_native(
        "Ground Station",
        Default::default(),
        Box::new(|_cc| Ok(Box::new(groundstation))),
    )
}

async fn simulated_telem(tx: mpsc::Sender<LogMessage>) {
    let start_time = Instant::now();
    loop {
        tx.send(simulate_message_log(&start_time)).await.ok();

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

fn simulate_message_log(start_time: &Instant) -> LogMessage {
    let host_timestamp = OffsetDateTime::now_utc()
    .unix_timestamp_nanos()
    .min(i64::MAX as i128) as i64;

    LogMessage {
        timestamp: format!("{:.9}", start_time.elapsed().as_secs_f64()),
        host_timestamp,
        level: Some(Level::Info),
        message: "Hello World!".to_string(),
        location: Some(LocationMessage {
            file_complete_path: PathBuf::from_str("src/bin/mock.rs").unwrap(),
            file: "bin/mock.rs".to_string(),
            line: 34,
            module_path: Some(ModulePath {
                crate_name: "groundstation".to_string(),
                modules: vec!["mock".to_string()],
                function: "simulated_telem".to_string(),
            }),
        }),
    }
}
