use std::{collections::BTreeMap, env, io::BufRead, path::PathBuf};

use anyhow::anyhow;
use circular_buffer::CircularBuffer;
use defmt_decoder::{DecodeError, Location, Table};
use tokio::{fs::{self}, sync::mpsc};

mod log_message;
pub use log_message::LogMessage;
mod source;
pub use source::Source;
mod elf_watcher;
pub use elf_watcher::ElfWatcher;

const READ_BUFFER_SIZE: usize = 1024;

pub struct DefmtParser {
    elf: PathBuf,
    table: Table,
    locs: Option<BTreeMap<u64, Location>>,
    elf_watcher: ElfWatcher,

    rx_source: mpsc::Receiver<Option<Source>>,

    source: Option<Source>,
    buf: CircularBuffer<1024, u8>,
    tx_defmt: mpsc::Sender<LogMessage>,
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
    
        log::debug!("absolute path of elf file with defmt messages: {:?}", elf);

        let (table, locs) = Self::load_elf(&elf).await?;

        let elf_watcher = ElfWatcher::new(elf.clone())?;

        Ok(Self {
            elf,
            table,
            locs,
            elf_watcher,
            tx_defmt,
            rx_source,
            source: None,
            buf: CircularBuffer::<1024, u8>::new(),
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            if self.elf_watcher.has_file_changed().await {
                (self.table, self.locs) = Self::load_elf(&self.elf).await?;
                log::info!("elf file changed; reloaded");
            }

            if let Some(source) = self.rx_source.recv().await {
                self.source = source;
                log::info!("source changed to {:?}", self.source);
            };

            self.handle_source().await?;
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

    async fn handle_source(&mut self) -> anyhow::Result<()> {
        let source = self.source.as_mut().ok_or_else(|| anyhow!("source not set"))?;
        let mut temp = [0; READ_BUFFER_SIZE/2];
        log::info!("listening for defmt messages");

        loop {
            // read from stdin or tcpstream and push it to the decoder
            let n = source.read(&mut temp).await?;
            self.buf.extend_from_slice(&temp[..n]);

            loop {
                match self.table.decode(self.buf.make_contiguous()) {
                    Ok((frame, consumed)) => {
                        let message = LogMessage::new(&frame, &self.locs);
                        log::info!("{:?}", message);
                        self.tx_defmt.send(message).await?;
                        self.buf.consume(consumed);
                    },
                    Err(DecodeError::UnexpectedEof) => break,
                    Err(DecodeError::Malformed) => match self.table.encoding().can_recover() {
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
}
