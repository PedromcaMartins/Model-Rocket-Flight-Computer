use eframe::egui;
use egui::TextStyle;
use tokio::sync::mpsc;

use crate::LogMessage;

#[derive(PartialEq)]
enum Sections {
    // Home,
    Terminal,
    // Graphs,
}

pub struct GroundStation {
    rx: mpsc::Receiver<LogMessage>,
    data: Vec<LogMessage>,
    section: Sections,
}

impl GroundStation {
    pub fn new(rx: mpsc::Receiver<LogMessage>) -> Self {
        Self { rx, data: Vec::new(), section: Sections::Terminal }
    }

    fn update_data(&mut self) {
        while let Ok(point) = self.rx.try_recv() {
            self.data.push(point);
        }
    }

    fn terminal(&mut self, ui: &mut egui::Ui) {        
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);

        egui::ScrollArea::both()
            .max_height(f32::INFINITY)
            .max_width(f32::INFINITY)
            .auto_shrink(false)
            .stick_to_bottom(true)
            .show_rows(
                ui,
                row_height,
                self.data.len(),
                |ui, row_range| {
                    for row in row_range {
                        ui.label(format!("{}\t {:?}", row + 1, self.data[row]));
                    }
                }
            );

        ui.ctx().request_repaint();
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
                    self.terminal(ui);
                }
            }
        });

        ctx.request_repaint(); // Keep updating UI
    }
}
