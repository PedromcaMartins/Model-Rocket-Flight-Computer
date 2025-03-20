use std::path::PathBuf;

use defmt_decoder::Location;
use defmt_parser::Level;
use groundstation::GroundStation;
use tokio::{sync::mpsc, time::Instant};

use groundstation::LogMessage;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (tx, rx) = mpsc::channel::<LogMessage>(100);

    tokio::spawn(simulated_telem(tx));

    // Spawn the GUI in a separate thread
    eframe::run_native(
        "Ground Station",
        Default::default(),
        Box::new(|_cc| Ok(Box::new(GroundStation::new(rx)))),
    )
}

async fn simulated_telem(tx: mpsc::Sender<LogMessage>) {
    let start_time = Instant::now();
    loop {
        let time = start_time.elapsed().as_nanos() as u64;
        let value = (time as f64 * 2.0).sin(); // Simulated telemetry data (sine wave)

        tx.send(
            LogMessage {
                timestamp: format!("{:.9}", start_time.elapsed().as_secs_f64()),
                level: Some(Level::Info),
                message: format!("Value: {}", value),
                location: Some(Location {
                    file: PathBuf::from("tests/mock_data.rs"),
                    line: 1,
                    module: "mock_data".to_string(),
                }),
            }
        ).await.ok();

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
