pub trait DeploymentSystem {
    type Error: core::fmt::Debug;

    async fn deploy(&mut self) -> Result<(), Self::Error>;

    /// Confirm that deployment was successful.
    /// Returns `Ok(true)` if deployed, `Ok(false)` if not yet confirmed.
    async fn verify_deployment(&mut self) -> Result<bool, Self::Error>;
}
