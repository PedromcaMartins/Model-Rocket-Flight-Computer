pub trait DeploymentSystem {
    type Error: core::fmt::Debug;

    fn deploy(&mut self) -> Result<(), Self::Error>;
}
