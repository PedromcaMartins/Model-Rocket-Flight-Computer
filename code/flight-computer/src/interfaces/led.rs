pub trait Led {
    type Error;

    async fn on(&mut self) -> Result<(), Self::Error>;
    async fn off(&mut self) -> Result<(), Self::Error>;
}
