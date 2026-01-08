// pub type Filename = &str;

pub trait FileSystem {
    type File;
    type Error: core::fmt::Debug;

    async fn exist_file(&mut self, filename: &str) -> Result<bool, Self::Error>;
    async fn create_file(&mut self, filename: &str) -> Result<Self::File, Self::Error>;

    async fn open_file_append(&mut self, filename: &str) -> Result<Self::File, Self::Error>;
    async fn close_file(&mut self, file: Self::File) -> Result<(), Self::Error>;

    async fn write_file(&mut self, file: &mut Self::File, data: &[u8]) -> Result<(), Self::Error>;
    async fn flush_file(&mut self, file: &mut Self::File) -> Result<(), Self::Error>;
}
