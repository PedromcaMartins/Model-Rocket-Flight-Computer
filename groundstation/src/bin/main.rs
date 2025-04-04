use groundstation::GroundStation;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    eframe::run_native(
        "Ground Station",
        Default::default(),
        Box::new(|cc| {
            Ok(Box::new(GroundStation::new(cc)))
        }),
    )
}
