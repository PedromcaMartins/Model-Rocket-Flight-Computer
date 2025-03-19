use groundstation::GroundStation;
use tokio::sync::mpsc;

use groundstation::LogMessage;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (_tx, rx) = mpsc::channel::<LogMessage>(100);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Ground Station",
        options,
        Box::new(|_cc| {
            Ok(Box::new(GroundStation::new(rx)))
        }),
    )
}
