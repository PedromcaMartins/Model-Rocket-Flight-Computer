use eframe::egui;
use tokio::sync::mpsc;

use crate::LogMessage;

mod terminal;
use terminal::Terminal;

#[derive(PartialEq, Default)]
enum Sections {
    // Home,
    #[default]
    Terminal,
    // Graphs,
}

pub struct GroundStation {
    rx: mpsc::Receiver<LogMessage>,
    data: Vec<LogMessage>,

    section: Sections,
    terminal: Terminal,
}

impl GroundStation {
    pub fn new(rx: mpsc::Receiver<LogMessage>) -> Self {
        Self { rx, data: Vec::new(), section: Default::default(), terminal: Default::default() }
    }

    fn update_data(&mut self) {
        while let Ok(point) = self.rx.try_recv() {
            self.data.push(point);
        }
    }
}

impl eframe::App for GroundStation {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_data();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.selectable_value(&mut self.section, Sections::Terminal, "Terminal");
            ui.separator();
            match self.section {
                Sections::Terminal => {
                    self.terminal.ui(ui, self.data.len(), &self.data);
                }
            }
        });

        ctx.request_repaint(); // Keep updating UI
    }
}
