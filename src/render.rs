use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::EguiContexts;
use bevy_pixel_buffer::bevy_egui::egui::Pos2;
use bevy_pixel_buffer::{bevy_egui::egui, prelude::*};

use crate::components::{GameTicks, GradientResource, Microphone, Source, SourceType, Wall};
use crate::constants::*;
use crate::grid::Grid;
use egui_plot::{Line, Plot, PlotPoints};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AttenuationType {
    Power,
    OriginalOneWay,
    Linear,
    Old,
    DoNothing,
}

#[derive(Resource)]
pub struct UiState {
    pub delta_l: f32,
    pub epsilon: f32,
    pub e_al: u32,
    pub render_abc_area: bool,
    pub at_type: AttenuationType,
    pub power_order: u32,
    pub image_rect_top: Pos2,
    pub test_mic: Vec<[f64; 2]>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            delta_l: 0.001,
            epsilon: 0.001,
            e_al: 50,
            render_abc_area: false,
            at_type: AttenuationType::Power,
            power_order: 5,
            image_rect_top: Pos2::default(),
            test_mic: vec![],
        }
    }
}

pub fn draw_egui(
    mut pb: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    mut sources: Query<&mut Source>,
    microphones: Query<&Microphone>,
    mut ui_state: ResMut<UiState>,
    mut grid: ResMut<Grid>,
) {
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
                    ui.set_min_width(ui.available_width());
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
                if ui
                    .add(egui::Slider::new(&mut ui_state.e_al, 2..=200).text("E_AL"))
                    .changed()
                {
                    grid.update_cells(ui_state.e_al);
                    let mut item = pb.iter_mut().next().expect("At least one pixel buffer");
                    item.pixel_buffer.size = PixelBufferSize {
                        size: if ui_state.render_abc_area {
                            UVec2::new(
                                SIMULATION_WIDTH + 2 * ui_state.e_al,
                                SIMULATION_HEIGHT + 2 * ui_state.e_al,
                            )
                        } else {
                            UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT)
                        },
                        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
                    };
                }

                if ui
                    .checkbox(&mut ui_state.render_abc_area, "Render Absorbing Boundary")
                    .clicked()
                {
                    let mut item = pb.iter_mut().next().expect("At least one pixel buffer");
                    item.pixel_buffer.size = PixelBufferSize {
                        size: if ui_state.render_abc_area {
                            UVec2::new(
                                SIMULATION_WIDTH + 2 * ui_state.e_al,
                                SIMULATION_HEIGHT + 2 * ui_state.e_al,
                            )
                        } else {
                            UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT)
                        },
                        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
                    };
                }

                egui::ComboBox::from_label("Attenuation Type")
                    .selected_text(format!("{:?}", ui_state.at_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut ui_state.at_type, AttenuationType::Power, "Power");
                        ui.selectable_value(
                            &mut ui_state.at_type,
                            AttenuationType::OriginalOneWay,
                            "OriginalOneWay",
                        );
                        ui.selectable_value(
                            &mut ui_state.at_type,
                            AttenuationType::Linear,
                            "Linear",
                        );
                        ui.selectable_value(
                            &mut ui_state.at_type,
                            AttenuationType::Old,
                            "Old (ThTank)",
                        );
                        ui.selectable_value(
                            &mut ui_state.at_type,
                            AttenuationType::DoNothing,
                            "Nothing",
                        );
                    });

                match ui_state.at_type {
                    AttenuationType::OriginalOneWay => ui.add(
                        egui::Slider::new(&mut ui_state.epsilon, 0.000001..=1.0)
                            .text("Epsilon")
                            .logarithmic(true),
                    ),
                    AttenuationType::Power => ui.add(
                        egui::Slider::new(&mut ui_state.power_order, 1..=10)
                            .text("Power")
                            .logarithmic(true),
                    ),
                    _other => ui.label("Nothing to change here"),
                }
            });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        // pb.update_fill_egui(ui.available_size());

        let texture = pb.egui_texture();
        let image = ui.image(egui::load::SizedTexture::new(texture.id, texture.size));
        ui_state.image_rect_top = image.rect.min;
    });

    // Very crude test, pls make better
    egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .default_height(200.0)
        .show(ctx, |ui| {
            ui.heading("Microphone Plot");
            ui.separator();
            //still need to enable a legend
            Plot::new("mic_plot").show(ui, |plot_ui| {
                for mic in microphones.iter() {
                    let points: PlotPoints = PlotPoints::new(mic.record.clone());
                    let line = Line::new(points);
                    plot_ui.line(line.name(format!("POS {}, {}", mic.x, mic.y)));
                }
            });
        });
}

pub fn draw_pixels(
    mut pb: QueryPixelBuffer,
    grid: Res<Grid>,
    gradient: Res<GradientResource>,
    ui_state: Res<UiState>,
) {
    let mut frame = pb.frame();
    frame.per_pixel_par(|coords, _| {
        let p = if ui_state.render_abc_area {
            grid.cells[Grid::coords_to_index(coords.x, coords.y, 8, ui_state.e_al)]
        } else {
            grid.cells[Grid::coords_to_index(
                coords.x + ui_state.e_al,
                coords.y + ui_state.e_al,
                8,
                ui_state.e_al,
            )]
        };
        let color = gradient.0.at((p) as f64);
        Pixel {
            r: (color.r * 255.) as u8,
            g: (color.g * 255.) as u8,
            b: (color.b * 255.) as u8,
            a: 255,
        }
    });
}

pub fn draw_walls(mut pb: QueryPixelBuffer, walls: Query<&Wall>, ui_state: Res<UiState>) {
    let mut frame = pb.frame();
    for wall in walls.iter() {
        let (x, y) = Grid::index_to_coords(wall.0 as u32, ui_state.e_al);
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
}

pub fn draw_misc() {}
