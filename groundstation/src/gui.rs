use std::path::PathBuf;

use eframe::egui;
use tokio::sync::mpsc;

use crate::LogMessage;

mod terminal;
use terminal::Terminal;

mod config;
use config::Config;

mod graphs;

#[derive(PartialEq, Default)]
enum Sections {
    // Home,
    /// The terminal section
    #[default]
    Terminal,
    /// The configuration section
    Config,
    // Graphs,
}

/// The Ground Station
/// 
/// Keeps track of the log messages and the GUI state
pub struct GroundStation {
    /// The receiver for the log messages
    rx: mpsc::Receiver<LogMessage>,
    /// The sender for the log messages
    tx: mpsc::Sender<LogMessage>,
    /// The log messages
    data: Vec<LogMessage>,

    /// The current section
    section: Sections,
    /// The terminal section
    terminal: Terminal,
    /// The configuration section
    config: Config,

    /// Defmt Parser
    /// 
    /// The path to the elf file
    elf: PathBuf,
    // source: Option<Source>,
}

impl Default for GroundStation {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel::<LogMessage>(100);
        Self {
            rx, 
            tx,
            data: Vec::new(), 
            section: Default::default(), 
            terminal: Default::default(), 
            config: Default::default(), 
            elf: Default::default(),
        }
    }
}

impl GroundStation {
    /// update the data using messages from the receiver
    fn update_data(&mut self) {
        while let Ok(point) = self.rx.try_recv() {
            self.data.push(point);
        }
    }

    /// Clone the sender for the log messages
    pub fn clone_tx(&self) -> mpsc::Sender<LogMessage> {
        self.tx.clone()
    }
}

impl eframe::App for GroundStation {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_data();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.section, Sections::Terminal, "Terminal");
                ui.selectable_value(&mut self.section, Sections::Config, "Config");
            });

            ui.separator();

            match self.section {
                Sections::Terminal => {
                    self.terminal.ui(ui, self.data.len(), &self.data);
                },
                Sections::Config => {
                    self.config.ui(ui);
                },
            }
        });

        ctx.request_repaint(); // Keep updating UI
    }
}
