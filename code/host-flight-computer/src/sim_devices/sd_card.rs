use std::{convert::Infallible, fs::OpenOptions, io::Write, path::PathBuf};

use flight_computer_lib::interfaces::{FileSystem, Filename};
use switch_hal::{InputSwitch, OutputSwitch};
use tokio::sync::watch;

/* -------------------------------------------------------------------------- */
/*                                 File System                                */
/* -------------------------------------------------------------------------- */

pub struct SimSdCard {
    dir_path: PathBuf,
}

impl SimSdCard {
    pub async fn new(dir_path: PathBuf) -> Self {
        tokio::fs::create_dir_all(&dir_path).await.expect("Failed to create log directory");
        Self { dir_path }
    }

    fn full_path(&self, filename: &Filename) -> PathBuf {
        self.dir_path.join(filename)
    }
}

impl FileSystem for SimSdCard {
    type File = std::fs::File;
    type Error = std::io::Error;

    fn exist_file(&mut self, filename: Filename) -> Result<bool, Self::Error> {
        std::fs::exists(self.full_path(&filename))
    }

    fn create_file(&mut self, filename: Filename) -> Result<Self::File, Self::Error> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.full_path(&filename))
    }

    fn open_file_append(&mut self, filename: Filename) -> Result<Self::File, Self::Error> {
        OpenOptions::new()
            .append(true)
            .create(false)
            .open(self.full_path(&filename))
    }

    fn close_file(&mut self, file: Self::File) -> Result<(), Self::Error> {
        drop(file);
        Ok(())
    }

    fn write_file(&mut self, file: &mut Self::File, data: &[u8]) -> Result<(), Self::Error> {
        file.write_all(data)
    }

    fn flush_file(&mut self, file: &mut Self::File) -> Result<(), Self::Error> {
        file.flush()
    }
}

/* -------------------------------------------------------------------------- */
/*                            Sd Card Detect Switch                           */
/* -------------------------------------------------------------------------- */


pub struct SimSdCardDetect {
    rx: watch::Receiver<bool>,
}

impl SimSdCardDetect {
    pub fn new(rx: watch::Receiver<bool>) -> Self {
        Self { rx }
    }
}

impl InputSwitch for SimSdCardDetect {
    type Error = Infallible;

    fn is_active(&mut self) -> Result<bool, Self::Error> {
        Ok(*self.rx.borrow())
    }
}


/* -------------------------------------------------------------------------- */
/*                             Sd Card Status Led                             */
/* -------------------------------------------------------------------------- */

pub struct SimSdCardStatusLed {
    tx: watch::Sender<bool>,
}

impl SimSdCardStatusLed {
    pub fn new(tx: watch::Sender<bool>) -> Self {
        Self { tx }
    }
}

impl OutputSwitch for SimSdCardStatusLed {
    type Error = watch::error::SendError<bool>;

    fn on(&mut self) -> Result<(), Self::Error> {
        self.tx.send(true)
    }

    fn off(&mut self) -> Result<(), Self::Error> {
        self.tx.send(false)
    }
}
