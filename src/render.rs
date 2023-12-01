use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::EguiContexts;
use bevy_pixel_buffer::{bevy_egui::egui, prelude::*};

use crate::components::{GradientResource, Source, SourceType, Wall};
use crate::constants::*;
use crate::grid::Grid;

#[derive(Resource)]
pub struct UiState {
    pub delta_l: f32,
    pub epsilon: f32,
    pub e_al: u32,
    pub render_abc_area: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            delta_l: 0.001,
            epsilon: 0.001,
            e_al: 50,
            render_abc_area: false,
        }
    }
}

pub fn draw_pixels(
    mut pb: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    grid: Res<Grid>,
    gradient: Res<GradientResource>,
    walls: Query<&Wall>,
    mut sources: Query<&mut Source>,
    mut ui_state: ResMut<UiState>,
) {
    let mut frame = pb.frame();
    frame.per_pixel_par(|coords, _| {
        let p = grid.cells[Grid::coords_to_index(coords.x + E_AL, coords.y + E_AL, 8)];
        // let p = grid.cells[Grid::coords_to_index(coords.x, coords.y, 8)]; // render abc
        let color = gradient.0.at((p) as f64);
        Pixel {
            r: (color.r * 255.) as u8,
            g: (color.g * 255.) as u8,
            b: (color.b * 255.) as u8,
            a: 255,
        }
    });
    // Walls
    for wall in walls.iter() {
        let (x, y) = Grid::index_to_coords(wall.0 as u32);
        //TODO: handle result
        let _ = frame.set(
            UVec2::new(x, y),
            Pixel {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );
    }

    let ctx = egui_context.ctx_mut();
    egui::SidePanel::left("left_panel")
        .default_width(300.)
        .show(ctx, |ui| {
            ui.spacing_mut().slider_width = 200.0;
            ui.heading("Settings");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(400.)
                .show(ui, |ui| {
                    for (index, mut s) in sources.iter_mut().enumerate() {
                        ui.collapsing(format!("Source {}", index), |ui| {
                            // debug ui
                            // ui.label(format!("Source {}", s.index));
                            // ui.label(format!("Type: {:?}", s.r#type));
                            // ui.label(format!("Amplitude: {}", s.amplitude));
                            // ui.label(format!("Frequency: {}", s.frequency));
                            // ui.label(format!("Phase: {}", s.phase));

                            ui.add(
                                egui::Slider::new(&mut s.frequency, 0.0..=20000.0)
                                    .text("Frequency (Hz)"),
                            );
                            ui.add(
                                egui::Slider::new(&mut s.amplitude, 0.0..=25.0).text("Amplitude"),
                            );
                            ui.add(egui::Slider::new(&mut s.phase, 0.0..=360.0).text("Phase (°)"));
                            egui::ComboBox::from_label("Waveform")
                                .selected_text(format!("{:?}", s.r#type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut s.r#type, SourceType::Sin, "Sinus");
                                    ui.selectable_value(&mut s.r#type, SourceType::Gauss, "Gauss");
                                });
                        });
                    }
                });

            ui.add(
                egui::Slider::new(&mut ui_state.delta_l, 0.0..=10.0)
                    .text("Delta L")
                    .logarithmic(true),
            );

            ui.collapsing("ABC", |ui| {
                ui.add(
                    egui::Slider::new(&mut ui_state.epsilon, 0.000001..=1.0)
                        .text("Epsilon")
                        .logarithmic(true),
                );
                ui.add(egui::Slider::new(&mut ui_state.e_al, 0..=200).text("E_AL"));
                ui.add(egui::widgets::Checkbox::new(
                    &mut ui_state.render_abc_area,
                    "Render Absorbing Boundary",
                ));
            });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        // pb.update_fill_egui(ui.available_size());

        let texture = pb.egui_texture();
        ui.image(egui::load::SizedTexture::new(texture.id, texture.size));
    });
}
