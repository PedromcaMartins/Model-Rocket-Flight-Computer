#[derive(Default)]
pub struct Graphs {

}

impl Graphs {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .min_height(32.0)
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Real-Time Data");
            });
        });

        egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(150.0)
        .width_range(80.0..=200.0)
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Accelerometer");
            });
        });

        egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(150.0)
        .width_range(80.0..=200.0)
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Gyroscope");
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Position in space / Altitude over time");
            });
        });
    }
}
