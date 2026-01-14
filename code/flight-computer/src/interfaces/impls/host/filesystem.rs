use std::path::PathBuf;
use tokio::fs::{self, OpenOptions, File};
use tokio::io::AsyncWriteExt;

use crate::interfaces::FileSystem;

/* -------------------------------------------------------------------------- */
/*                                 File System                                */
/* -------------------------------------------------------------------------- */

pub struct HostFileSystem {
    dir_path: PathBuf,
}

impl HostFileSystem {
    pub async fn new(dir_path: PathBuf) -> Self {
        tokio::fs::create_dir_all(&dir_path).await.expect("Failed to create log directory");
        Self { dir_path }
    }

    fn full_path(&self, filename: &str) -> PathBuf {
        self.dir_path.join(filename)
    }
}

impl FileSystem for HostFileSystem {
    type File = File;
    type Error = std::io::Error;

    async fn exist_file(&mut self, filename: &str) -> Result<bool, Self::Error> {
        let path = self.full_path(filename);

        fs::try_exists(path).await
    }

    async fn create_file(&mut self, filename: &str) -> Result<Self::File, Self::Error> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.full_path(filename))
            .await
    }

    async fn open_file_append(&mut self, filename: &str) -> Result<Self::File, Self::Error> {
        OpenOptions::new()
            .append(true)
            .create(false)
            .open(self.full_path(filename))
            .await
    }

    async fn close_file(&mut self, file: Self::File) -> Result<(), Self::Error> {
        drop(file);
        Ok(())
    }

    async fn write_file(&mut self, file: &mut Self::File, data: &[u8]) -> Result<(), Self::Error> {
        file.write_all(data).await
    }

    async fn flush_file(&mut self, file: &mut Self::File) -> Result<(), Self::Error> {
        file.flush().await
    }
}
