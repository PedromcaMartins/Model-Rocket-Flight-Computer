use std::{env, path::PathBuf};

use groundstation::{defmt_parser::{handle_stream, list_ports, Source}, GroundStation};
use tokio::sync::mpsc;

use groundstation::LogMessage;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (tx, _rx) = mpsc::channel::<LogMessage>(100);

    list_ports().unwrap();

    // We create the source outside of the run command since recreating the stdin looses us some frames
    let mut source = Source::serial(PathBuf::from("COM4"), 115200).unwrap();
    let mut manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let _ = manifest_dir.pop();
    let elf = manifest_dir
        .join("target")
        .join("thumbv7em-none-eabihf")
        .join("debug")
        .join("flight-computer");

    log::debug!("absolute path of elf file with defmt messages: {:?}", elf);
    tokio::spawn(async move {
        handle_stream(elf, &mut source, tx).await
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Ground Station",
        options,
        Box::new(|_cc| {
            Ok(Box::new(GroundStation::default()))
        }),
    )
}
