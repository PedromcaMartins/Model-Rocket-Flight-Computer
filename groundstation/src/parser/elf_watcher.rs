use std::path::PathBuf;

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

pub struct ElfWatcher {
    pub path: PathBuf,
    rx: mpsc::Receiver<Result<Event, notify::Error>>,
}

impl ElfWatcher {
    pub fn new(elf: PathBuf) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel(1);
    
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

        Ok(Self {
            path,
            rx,
        })
    }

    /// Blocks until the elf file has changed
    /// 
    /// Useful for reloading the elf file
    pub async fn has_file_changed(&mut self) -> bool {
        loop {
            if let Some(Ok(event)) = self.rx.recv().await {
                if event.paths.contains(&self.path) {
                    if let notify::EventKind::Create(_) | notify::EventKind::Modify(_) = event.kind {
                        break;
                    }
                }
            }
        }
        true
    }
}
