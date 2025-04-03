use std::{path::PathBuf, time::Duration};

use anyhow::anyhow;
use tokio::{fs::File, io::AsyncReadExt, net::TcpStream};
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};

#[derive(Debug)]
pub enum Source {
    File(File),
    Tcp(TcpStream),
    Serial(SerialStream),
}

impl Source {
    pub async fn file(path: PathBuf) -> anyhow::Result<Self> {
        match File::open(path).await {
            Ok(file) => Ok(Source::File(file)),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub async fn tcp(host: String, port: u16) -> anyhow::Result<Self> {
        match TcpStream::connect((host, port)).await {
            Ok(stream) => Ok(Source::Tcp(stream)),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn serial(path: PathBuf, baud: u32) -> anyhow::Result<Self> {
        let mut ser = tokio_serial::new(path.to_string_lossy(), baud).open_native_async()?;
        ser.set_timeout(Duration::from_millis(500))?;
        Ok(Source::Serial(ser))
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<usize> {
        match self {
            Source::File(file) => Ok(file.read(buf).await?),
            Source::Tcp(tcpstream) => Ok(tcpstream.read(buf).await?),
            Source::Serial(serial) => Ok(serial.read(buf).await?),
        }
    }
}
