use egui_plotter::{EguiBackend, DEFAULT_MOVE_SCALE, DEFAULT_SCROLL_SCALE};
use log::info;
use nalgebra::{Quaternion, UnitQuaternion, Vector3};
use telemetry::{AltimeterMessage, GpsMessage, ImuMessage};
use plotters::prelude::*;

pub struct MouseData {
    chart_pitch: f32,
    chart_yaw: f32,
    chart_scale: f32,
    chart_pitch_vel: f32,
    chart_yaw_vel: f32,
}

impl Default for MouseData {
    fn default() -> Self {
        Self {
            chart_pitch: 0.5,
            chart_yaw: 0.5,
            chart_scale: 0.5,
            chart_pitch_vel: 0.0,
            chart_yaw_vel: 0.0,
        }
    }
}

impl MouseData {
    pub fn update_mouse_data(&mut self, ui: &mut egui::Ui) {
        let panel_rect = ui.max_rect(); // Get the bounding rectangle of the panel

        let (pitch_delta, yaw_delta, scale_delta) = ui.input(|input| {
            let pointer = &input.pointer;
            let delta = pointer.delta();

            // Check if the cursor is inside the panel
            let is_inside_panel = panel_rect.contains(pointer.interact_pos().unwrap_or_default());

            let (pitch_delta, yaw_delta) = match pointer.primary_down() && is_inside_panel {
                true => (delta.y * DEFAULT_MOVE_SCALE, -delta.x * DEFAULT_MOVE_SCALE),
                false => (self.chart_pitch_vel * DEFAULT_MOVE_SCALE, self.chart_yaw_vel * DEFAULT_MOVE_SCALE),
            };

            let scale_delta = if is_inside_panel { input.raw_scroll_delta.y * DEFAULT_SCROLL_SCALE } else { 0.0 };

            (pitch_delta, yaw_delta, scale_delta)
        });

        self.chart_pitch_vel = pitch_delta;
        self.chart_yaw_vel = yaw_delta;

        self.chart_pitch += self.chart_pitch_vel;
        self.chart_yaw += self.chart_yaw_vel;
        self.chart_scale += scale_delta;
    }
}

#[derive(Default)]
pub struct PositionChart {
    mouse_data: MouseData,
}

impl PositionChart {
    pub fn plot(
        &mut self, 
        ui: &mut egui::Ui, 
        altimeter_data: &[AltimeterMessage], 
        gps_data: &[GpsMessage],
        imu_data: &[ImuMessage],
    ) {
        let mut positions  = gps_data
            .iter()
            .zip(altimeter_data.iter())
            .rev()
            .take(120)
            .map(|(gps, altimeter)| {
                let lon = gps.longitude;
                let lat = gps.latitude;
                let alt = altimeter.altitude as f64;

                (lon, alt, lat)
            });

        let current_position = positions
            .next()
            .map(|(x, y, z)| {
                Vector3::from_vec([x, y, z].into())
            });

        let current_orientation = imu_data
            .last()
            .map(|imu| {
                Quaternion::from_vector([
                    imu.quaternion[0] as f64,
                    imu.quaternion[1] as f64,
                    imu.quaternion[2] as f64,
                    imu.quaternion[3] as f64,
                ].into())
            });

        self.mouse_data.update_mouse_data(ui);

        // Next plot everything
        let root = EguiBackend::new(ui).into_drawing_area();

        root.fill(&WHITE).unwrap();

        let axis = (-3.0..3.0).step(0.1);

        // create a 3d chart
        let mut chart = ChartBuilder::on(&root)
            .caption("Position in space", (FontFamily::SansSerif, 20))
            .build_cartesian_3d(axis.clone(), axis.clone(), axis)
            .unwrap();

        // use mouse data to set projection
        chart.with_projection(|mut pb| {
            pb.yaw = self.mouse_data.chart_yaw as f64;
            pb.pitch = self.mouse_data.chart_pitch as f64;
            pb.scale = self.mouse_data.chart_scale as f64;
            pb.into_matrix()
        });

        // display the axes
        chart
            .configure_axes()
            .light_grid_style(BLACK.mix(0.15))
            .max_light_lines(3)
            .draw()
            .unwrap();

        // draw the trajectory from last positions
        chart
            .draw_series(LineSeries::new(
                positions,
                &BLACK,
            ))
            .unwrap()
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK));

        // create the axes styles
        let axis_style_x = ShapeStyle {
            color: RED.to_rgba(),
            filled: false,
            stroke_width: 2,
        };

        let axis_style_y = ShapeStyle {
            color: GREEN.to_rgba(),
            filled: false,
            stroke_width: 2,
        };

        let axis_style_z = ShapeStyle {
            color: BLUE.to_rgba(),
            filled: false,
            stroke_width: 2,
        };

        // if there's no position information, don't draw the world axys
        if current_position.is_some() {
            // draw the world axys
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)],
                axis_style_x,
            ))).unwrap();

            chart.draw_series(std::iter::once(PathElement::new(
                vec![(0.0, 0.0, 0.0), (0.0, 0.0, 1.0)],
                axis_style_y,
            ))).unwrap();

            chart.draw_series(std::iter::once(PathElement::new(
                vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
                axis_style_z,
            ))).unwrap();
        }

        // draw body axys according to position and orientation
        let (body_vector_x, body_vector_y, body_vector_z) = match current_orientation {
            Some(orientation) => get_body_frame_vectors(orientation),
            None => (Vector3::x(), Vector3::y(), Vector3::z()),
        };

        let current_position = match current_position {
            Some(pos) => pos,
            None if altimeter_data.is_empty() => {
                info!("No position data available");
                Vector3::default()
            },
            _ => {
                let alt = altimeter_data.last().unwrap().altitude as f64;
                info!("Using altimeter data for position, altitude: {:?}", alt);
                Vector3::new(0.0, 0.0, alt)
            }
        };

        let body_vector_x = current_position + body_vector_x;
        let body_vector_y = current_position + body_vector_y;
        let body_vector_z = current_position + body_vector_z;

        let current_position = (current_position.x, current_position.y, current_position.z);

        chart.draw_series(std::iter::once(PathElement::new(
            vec![
                current_position,
                (body_vector_x.x, body_vector_x.y, body_vector_x.z),
            ],
            axis_style_x,
        ))).unwrap();

        chart.draw_series(std::iter::once(PathElement::new(
            vec![
                current_position,
                (body_vector_y.x, body_vector_y.y, body_vector_y.z),
            ],
            axis_style_y,
        ))).unwrap();

        chart.draw_series(std::iter::once(PathElement::new(
            vec![
                current_position,
                (body_vector_z.x, body_vector_z.y, body_vector_z.z),
            ],
            axis_style_z,
        ))).unwrap();

        root.present().unwrap();
    }
}

/// Returns the body frame unit vectors (X, Y, Z) in world coordinates,
/// given a quaternion orientation.
fn get_body_frame_vectors(
    orientation: Quaternion<f64>,
) -> (Vector3<f64>, Vector3<f64>, Vector3<f64>) {
    // Convert to a UnitQuaternion to safely use rotation
    let rotation = UnitQuaternion::from_quaternion(orientation);

    // Standard basis vectors in body frame
    let x_body = Vector3::x();
    let y_body = Vector3::y();
    let z_body = Vector3::z();

    // Rotate the basis vectors using the quaternion
    let x_world = rotation.transform_vector(&x_body);
    let y_world = rotation.transform_vector(&y_body);
    let z_world = rotation.transform_vector(&z_body);

    (x_world, y_world, z_world)
}
