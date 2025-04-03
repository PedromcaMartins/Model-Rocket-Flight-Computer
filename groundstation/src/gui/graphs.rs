use egui_plot::{Line, Plot};
use telemetry::{AltimeterMessage, GpsMessage, ImuMessage};

#[derive(Default)]
pub struct Graphs {
}

impl Graphs {
    pub fn ui(
        &mut self, 
        ui: &mut egui::Ui, 
        imu_data: &[ImuMessage], 
        altimeter_data: &[AltimeterMessage], 
        gps_data: &[GpsMessage]
    ) {
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
            self.acceleration_plot(ui, imu_data);
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

    fn acceleration_plot(
        &mut self, 
        ui: &mut egui::Ui, 
        imu_data: &[ImuMessage]
    ) {
        ui.vertical_centered(|ui| {
            ui.label("Acceleration X");
            Plot::new("Acceleration X")
            .legend(Default::default())
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new(imu_data
                    .iter()
                    .rev()
                    .take(25)
                    .map(|data| {
                        let x = (data.timestamp as f64) / 1_000_000_f64;
                        let y = data.acceleration[0] as f64;
                        [x, y]
                    })
                    .collect::<Vec<_>>()
                ).name("X-Axis"));
            });
        });
    }
}
