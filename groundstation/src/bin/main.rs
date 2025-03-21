use groundstation::GroundStation;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let groundstation = GroundStation::default();

    eframe::run_native(
        "Ground Station",
        Default::default(),
        Box::new(|_cc| {
            Ok(Box::new(groundstation))
        }),
    )
}
