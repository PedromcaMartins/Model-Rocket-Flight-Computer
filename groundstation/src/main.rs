use gui::{simulated_telem, MyApp};
use tokio::sync::mpsc;

mod defmt_parser;
mod gui;

type Message = String;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (tx, rx) = mpsc::channel::<Message>(100);

    // Simulated telemetry data sender (runs in the background)
    tokio::spawn(simulated_telem(tx));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Ground Station",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp::new(rx)))
        }),
    )
}
