pub trait Led {
    type Error: core::fmt::Debug;

    async fn on(&mut self) -> Result<(), Self::Error>;
    async fn off(&mut self) -> Result<(), Self::Error>;
    async fn toggle(&mut self) -> Result<(), Self::Error>;
}
