use std::{collections::BTreeMap, env, io::BufRead, path::PathBuf};

use anyhow::anyhow;
use circular_buffer::CircularBuffer;
use defmt_decoder::{DecodeError, Location, Table};
use tokio::{fs::{self}, select, sync::mpsc};

mod log_message;
pub use log_message::{LogMessage, ModulePath, Location as LocationMessage};
mod source;
pub use source::Source;
mod elf_watcher;
pub use elf_watcher::ElfWatcher;

const READ_BUFFER_SIZE: usize = 1024;

struct SourceHandler {
    table: Table,
    locs: Option<BTreeMap<u64, Location>>,

    source: Option<Source>,
    buf: CircularBuffer<READ_BUFFER_SIZE, u8>,
}

impl SourceHandler {
    async fn run(&mut self) -> anyhow::Result<LogMessage> {
        let source = self.source.as_mut().ok_or_else(|| anyhow!("source not set"))?;
        let mut temp = [0; READ_BUFFER_SIZE/2];

        let mut current_dir = env::current_dir()?; 
        let _ = current_dir.pop();
        current_dir.push("flight-computer");
        current_dir.push("src");

        log::info!("listening for defmt messages");

        loop {
            // read from stdin or tcpstream and push it to the decoder
            let n = source.read(&mut temp).await?;
            self.buf.extend_from_slice(&temp[..n]);

            match self.table.decode(self.buf.make_contiguous()) {
                Ok((frame, consumed)) => {
                    let message = LogMessage::new(&frame, &self.locs, &current_dir);
                    log::info!("{:?}", message);
                    self.buf.consume(consumed);
                    return Ok(message);
                },
                Err(DecodeError::UnexpectedEof) => continue,
                Err(DecodeError::Malformed) => match self.table.encoding().can_recover() {
                    // if recovery is impossible, abort
                    false => {
                        log::error!("malformed frame; impossible to recover");
                        return Err(DecodeError::Malformed.into())
                    },
                    // if recovery is possible, skip the current frame and continue with new data
                    true => {
                        log::warn!("malformed frame skipped");
                        continue;
                    }
                },
            }
        }
    }
}

pub struct DefmtParser {
    elf_watcher: ElfWatcher,

    tx_defmt: mpsc::Sender<LogMessage>,
    rx_source: mpsc::Receiver<Option<Source>>,

    source_handler: SourceHandler,
}

impl DefmtParser {
    pub async fn new(tx_defmt: mpsc::Sender<LogMessage>, rx_source: mpsc::Receiver<Option<Source>>) -> anyhow::Result<Self> {
        let mut manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let _ = manifest_dir.pop();
        let elf = manifest_dir
            .join("target")
            .join("thumbv7em-none-eabihf")
            .join("debug")
            .join("flight-computer");

        log::info!("absolute path of elf file with defmt messages: {:?}", elf);

        let (table, locs) = Self::load_elf(&elf).await?;

        Ok(Self {
            elf_watcher: ElfWatcher::new(elf)?,
            tx_defmt,
            rx_source,
            source_handler: SourceHandler {
                table,
                locs,
                source: None,
                buf: CircularBuffer::<READ_BUFFER_SIZE, u8>::new(),
            },
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            select! {
                changed = self.elf_watcher.has_file_changed() => {
                    if changed {
                        (self.source_handler.table, self.source_handler.locs) = Self::load_elf(&self.elf_watcher.path).await?;
                        log::info!("elf file changed; reloaded");
                    }
                }
                source = self.rx_source.recv() => {
                    if let Some(source) = source {
                        self.source_handler.source = source;
                        log::info!("source changed to {:?}", self.source_handler.source);
                    }
                }
                message = self.source_handler.run() => {
                    if let Ok(message) = message {
                        self.tx_defmt.send(message).await?;
                    }
                }
            }
        }
    }

    async fn load_elf(elf: &PathBuf) -> anyhow::Result<(Table, Option<BTreeMap<u64, Location>>)> {
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

        Ok((table, locs))
    }
}
