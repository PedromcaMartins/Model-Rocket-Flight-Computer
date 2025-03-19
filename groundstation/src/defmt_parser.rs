use std::{path::{Path, PathBuf}, time::Duration};

use anyhow::anyhow;
use defmt_decoder::{DecodeError, Frame, Location, Locations, Table};
use tokio::{fs, io::{self, AsyncReadExt, Stdin}, net::TcpStream, sync::mpsc::Sender};
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};

#[derive(Debug)]
pub struct LogMessage {
    pub timestamp: String,
    pub level: Option<defmt_parser::Level>,
    pub message: String,
    pub location: Option<Location>, 
}

impl LogMessage {
    pub fn new(frame: &Frame, locs: &Option<Locations>) -> Self {
        let location = locs.as_ref()
        .and_then(|locs| locs.get(&frame.index()))
        .map(|loc| {
            let path = to_relative_from_src(&loc.file).unwrap_or_else(|| loc.file.clone());
            Location {
                file: path,
                line: loc.line,
                module: loc.module.clone(),
            }
        });

        Self {
            timestamp: frame
                .display_timestamp()
                .map(|ts| ts.to_string())
                .unwrap_or_default(),
            level: frame.level(),
            message: frame.display_message().to_string(),
            location,
        }
    }
}

pub enum Source {
    Stdin(Stdin),
    Tcp(TcpStream),
    Serial(SerialStream),
}

impl Source {
    pub fn stdin() -> Self {
        Source::Stdin(io::stdin())
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
            Source::Stdin(stdin) => Ok(stdin.read(buf).await?),
            Source::Tcp(tcpstream) => Ok(tcpstream.read(buf).await?),
            Source::Serial(serial) => Ok(serial.read(buf).await?),
        }
    }
}

const READ_BUFFER_SIZE: usize = 1024;

pub async fn handle_stream(elf: PathBuf, source: &mut Source, tx: Sender<LogMessage>) -> anyhow::Result<()> {
    let bytes = fs::read(elf).await?;
    let table = Table::parse(&bytes)?.ok_or_else(|| anyhow!(".defmt data not found"))?;
    let locs = table.get_locations(&bytes)?;

    // check if the locations info contains all the indicies
    let locs = if table.indices().all(|idx| locs.contains_key(&(idx as u64))) {
        Some(locs)
    } else {
        log::warn!("(BUG) location info is incomplete; it will be omitted from the output");
        None
    };

    let mut buf = [0; READ_BUFFER_SIZE];
    log::info!("listening for defmt messages");

    loop {
        // read from stdin or tcpstream and push it to the decoder
        let n = source.read(&mut buf).await?;
        let mut start = 0;

        loop {
            match table.decode(&buf[start..n]) {
                Ok((frame, consumed)) => {
                    let message = LogMessage::new(&frame, &locs);
                    log::info!("{:?}", message);
                    tx.send(message).await?;

                    start += consumed;
                },
                Err(DecodeError::UnexpectedEof) => break,
                Err(DecodeError::Malformed) => match table.encoding().can_recover() {
                    // if recovery is impossible, abort
                    false => {
                        log::error!("malformed frame; impossible to recover");
                        return Err(DecodeError::Malformed.into())
                    },
                    // if recovery is possible, skip the current frame and continue with new data
                    true => {
                        log::warn!("malformed frame skipped");
                        break;
                    }
                },
            }
        }
    }
}


/* -------------------------------------------------------------------------- */
/*                                   Extras                                   */
/* -------------------------------------------------------------------------- */

use tokio::{
    select,
    sync::mpsc::Receiver,
};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

pub fn list_ports() -> anyhow::Result<()> {
    let ports = tokio_serial::available_ports()?;
    if ports.is_empty() {
        println!("No COM ports found.");
    } else {
        println!("Available COM Ports:");
        for port in ports {
            println!(" - {}", port.port_name);
        }
    }
    Ok(())
}

pub async fn run_and_watch(elf: PathBuf, mut source: Source, tx_log: Sender<LogMessage>) -> anyhow::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let path = elf.clone().canonicalize().unwrap();

    // We want the elf directory instead of the elf, since some editors remove
    // and recreate the file on save which will remove the notifier
    let directory_path = path.parent().unwrap();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.blocking_send(res);
        },
        Config::default(),
    )?;
    watcher.watch(directory_path.as_ref(), RecursiveMode::NonRecursive)?;

    loop {
        select! {
            r = handle_stream(elf.clone(), &mut source, tx_log.clone()) => r?,
            _ = has_file_changed(&mut rx, &path) => ()
        }
    }
}

async fn has_file_changed(rx: &mut Receiver<Result<Event, notify::Error>>, path: &PathBuf) -> bool {
    loop {
        if let Some(Ok(event)) = rx.recv().await {
            if event.paths.contains(path) {
                if let notify::EventKind::Create(_) | notify::EventKind::Modify(_) = event.kind {
                    break;
                }
            }
        }
    }
    true
}

fn to_relative_from_src(absolute_path: &Path) -> Option<PathBuf> {
    // Find "src" in the path components
    for (i, component) in absolute_path.components().enumerate() {
        if component.as_os_str() == "src" {
            // Create a relative path after "src/"
            return Some(absolute_path.iter().skip(i + 1).collect());
        }
    }
    
    None // "src" not found in the path
}
