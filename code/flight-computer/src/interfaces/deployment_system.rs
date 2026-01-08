pub trait DeploymentSystem {
    type Error: core::fmt::Debug;

    async fn deploy(&mut self) -> Result<(), Self::Error>;
}
