//! NDJSON record storage for FC telemetry.
//!
//! Writes received `Record`s to a session file at
//! `logs/gs_records/<timestamp>/records.ndjson` and keeps an in-memory
//! cache for REST API reads. The file is append-only within a session.

use std::io::Write;

use chrono::Local;
use serde::Serialize;
use tracing::info;

use crate::config::Config;

/// Manages the NDJSON output file + in-memory record cache for one GS session.
pub struct RecordStorage {
    file: std::io::BufWriter<std::fs::File>,
    /// Number of records written so far.
    count: u64,
    /// Human-readable session start time.
    session_start: String,
    /// In-memory cache of all records for REST API reads.
    records: Vec<proto::record::Record>,
}

impl RecordStorage {
    /// Create a new storage session, creating the output directory and file.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file
    /// cannot be opened for writing.
    pub fn create() -> anyhow::Result<Self> {
        let ts = Local::now()
            .format(Config::RECORDS_TIMESTAMP_FORMAT)
            .to_string();
        let dir = Config::records_root_dir().join(&ts);
        std::fs::create_dir_all(&dir)?;

        let path = dir.join("records.ndjson");
        let file = std::fs::File::create(&path)?;
        let file = std::io::BufWriter::new(file);

        info!(path = %path.display(), "record storage opened");

        Ok(Self {
            file,
            count: 0,
            session_start: ts,
            records: Vec::new(),
        })
    }

    /// Append one serialisable record as a JSON line.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails (disk full, permissions, etc.).
    /// The caller should log and continue — storage failures are non-fatal
    /// for the GS process.
    pub fn append<T: Serialize>(&mut self, record: &T) -> anyhow::Result<()> {
        let mut line = serde_json::to_vec(record)?;
        line.push(b'\n');
        self.file.write_all(&line)?;
        self.file.flush()?;
        self.count = self.count.saturating_add(1);
        Ok(())
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn session_start(&self) -> &str {
        &self.session_start
    }

    /// Store a telemetry record: write to NDJSON and retain in memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the NDJSON write fails. The in-memory cache is
    /// updated regardless, so a failed write is non-fatal for reads.
    pub fn store_record(&mut self, record: proto::record::Record) -> anyhow::Result<()> {
        // In-memory cache (append even if NDJSON write fails).
        self.records.push(record.clone());
        // NDJSON write.
        self.append(&record)
    }

    /// All records from the current session, in arrival order.
    pub fn records(&self) -> &[proto::record::Record] {
        &self.records
    }

    /// The most recent record, if any.
    pub fn latest_record(&self) -> Option<&proto::record::Record> {
        self.records.last()
    }
}
