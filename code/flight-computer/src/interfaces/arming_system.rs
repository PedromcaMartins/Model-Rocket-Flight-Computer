pub trait ArmingSystem {
    type Error;

    async fn wait_arm(&mut self) -> Result<(), Self::Error>;
}
