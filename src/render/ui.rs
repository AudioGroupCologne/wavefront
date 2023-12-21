use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::epaint::CircleShape;
use bevy_pixel_buffer::bevy_egui::egui::{pos2, Color32, Frame, Margin, Stroke, Vec2};
use bevy_pixel_buffer::bevy_egui::{egui, EguiContexts};
use bevy_pixel_buffer::prelude::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::components::microphone::*;
use crate::components::source::*;
use crate::components::wall::{Overlay, WallBlock};
use crate::grid::grid::Grid;
use crate::math::constants::*;
use crate::math::fft::calc_mic_spectrum;
use crate::math::transformations::f32_map_range;
use crate::render::state::*;

pub fn draw_egui(
    mut pixel_buffers: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    mut sources: Query<&mut Source>,
    wallblocks: Query<&WallBlock, Without<Overlay>>,
    mut microphones: Query<&mut Microphone>,
    mut ui_state: ResMut<UiState>,
    diagnostics: Res<DiagnosticsStore>,
    mut grid: ResMut<Grid>,
    images: Local<Images>,
) {
    let cursor_icon = egui_context.add_image(images.cursor_icon.clone_weak());
    let ctx = egui_context.ctx_mut();

    // Side Panel (Sources, Mic, Tool Options, Settings)
    egui::SidePanel::left("left_panel")
        .default_width(450.)
        .show(ctx, |ui| {
            ui.spacing_mut().slider_width = 200.0;

            ui.heading("Settings");
            if let Some(value) = diagnostics
                .get(FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.smoothed())
            {
                ui.label(format!("FPS: {:.1}", value));
            }

            ui.separator();

            // Sources
            egui::ScrollArea::vertical()
                .id_source("source_scroll_area")
                .max_height(400.)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    for (index, mut s) in sources.iter_mut().enumerate() {
                        ui.collapsing(format!("Source {}", index), |ui| {
                            ui.add(
                                egui::Slider::new(&mut s.frequency, 0.0..=20000.0)
                                    .text("Frequency (Hz)"),
                            );
                            ui.add(
                                egui::Slider::new(&mut s.amplitude, 0.0..=25.0).text("Amplitude"),
                            );
                            ui.add(egui::Slider::new(&mut s.phase, 0.0..=360.0).text("Phase (°)"));
                            egui::ComboBox::from_label("Waveform")
                                .selected_text(format!("{:?}", s.source_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut s.source_type,
                                        SourceType::Sin,
                                        "Sinus",
                                    );
                                    ui.selectable_value(
                                        &mut s.source_type,
                                        SourceType::Gauss,
                                        "Gauss",
                                    );
                                    ui.selectable_value(
                                        &mut s.source_type,
                                        SourceType::WhiteNoise,
                                        "White Noise",
                                    );
                                });
                        });
                    }
                });

            ui.separator();

            // Microphones
            egui::ScrollArea::vertical()
                .id_source("mic_scroll_area")
                .max_height(400.)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    for mut s in microphones.iter_mut() {
                        ui.collapsing(format!("Microphone {}", s.id), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("x:");
                                ui.add(
                                    egui::DragValue::new(&mut s.x)
                                        .speed(1)
                                        .clamp_range(0.0..=SIMULATION_WIDTH as f32),
                                );
                                ui.add_space(10.);
                                ui.label("y:");
                                ui.add(
                                    egui::DragValue::new(&mut s.y)
                                        .speed(1)
                                        .clamp_range(0.0..=SIMULATION_HEIGHT as f32),
                                );
                            });
                        });
                    }
                });

            // General Settings
            egui::TopBottomPanel::bottom("general_settings_bottom_panel").show_inside(ui, |ui| {
                ui.heading("General Settings");
                ui.separator();

                if ui
                    .button(if ui_state.is_running { "Stop" } else { "Start" })
                    .clicked()
                {
                    ui_state.is_running = !ui_state.is_running;
                }

                if ui.button("Reset").clicked() {
                    grid.update_cells(ui_state.e_al);
                    for mut mic in microphones.iter_mut() {
                        mic.clear();
                    }
                }

                ui.add(
                    egui::Slider::new(&mut ui_state.delta_l, 0.0..=10.0)
                        .text("Delta L")
                        .logarithmic(true),
                );

                if ui
                    .checkbox(&mut ui_state.show_plots, "Show Plots")
                    .clicked()
                {
                    for mut mic in microphones.iter_mut() {
                        mic.clear();
                    }
                }

                // ABC
                ui.collapsing("ABC", |ui| {
                    if ui
                        .add(egui::Slider::new(&mut ui_state.e_al, 2..=200).text("E_AL"))
                        .changed()
                    {
                        grid.update_cells(ui_state.e_al);
                        let mut pb = pixel_buffers.iter_mut().next().expect("one pixel buffer");
                        pb.pixel_buffer.size = PixelBufferSize {
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
                        let mut pb = pixel_buffers.iter_mut().next().expect("one pixel buffer");

                        pb.pixel_buffer.size = PixelBufferSize {
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
                            ui.selectable_value(
                                &mut ui_state.at_type,
                                AttenuationType::Power,
                                "Power",
                            );
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
                                "Old (ThTank) NOT WORKING",
                            );
                            ui.selectable_value(
                                &mut ui_state.at_type,
                                AttenuationType::DoNothing,
                                "Nothing NOT WORKING",
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

                ui.add_space(10.);
            });

            // Tool Options
            egui::TopBottomPanel::bottom("tool_options_panel").show_inside(ui, |ui| {
                ui.heading("Tool Options");
                ui.separator();

                ui.set_enabled(!ui_state.render_abc_area);

                match ui_state.current_tool {
                    ToolType::PlaceSource => {}
                    ToolType::MoveSource => {}
                    ToolType::MoveWall => {}
                    ToolType::DrawWall => {
                        egui::ComboBox::from_label("Select Brush Type")
                            .selected_text(format!("{:?}", ui_state.wall_brush))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.selectable_value(
                                    &mut ui_state.wall_brush,
                                    WallBrush::Rectangle,
                                    "Rectangle",
                                );
                                ui.selectable_value(
                                    &mut ui_state.wall_brush,
                                    WallBrush::CircleBrush,
                                    "Brush",
                                );
                            });
                        if ui_state.wall_brush == WallBrush::CircleBrush {
                            ui.add(
                                egui::Slider::new(&mut ui_state.wall_brush_radius, 1..=100)
                                    .text("Brush Radius"),
                            );
                        }
                        ui.add(
                            egui::Slider::new(&mut ui_state.wall_reflection_factor, 0.0..=1.0)
                                .text("Wall Reflection Factor"),
                        );
                    }
                    ToolType::ResizeWall => {}
                }

                ui.add_space(10.);
            });
        });

    // FFT Heatmap
    if ui_state.plot_type == PlotType::FrequencyDomain && ui_state.show_plots {
        egui::SidePanel::right("spectrum_panel")
            .frame(Frame::default().inner_margin(Margin {
                left: 0.,
                right: 0.,
                top: 0.,
                bottom: 0.,
            }))
            .default_width(400.)
            .resizable(false)
            .show(ctx, |ui| {
                ui_state.spectrum_size = ui.available_size();
                let mut pb = pixel_buffers
                    .iter_mut()
                    .nth(1)
                    .expect("second pixel buffer");

                let texture = pb.egui_texture();
                ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(texture.id, texture.size))
                        .shrink_to_fit(),
                );

                pb.pixel_buffer.size = PixelBufferSize {
                    size: UVec2::new(
                        ui_state.spectrum_size.x as u32,
                        ui_state.spectrum_size.y as u32,
                    ),

                    pixel_size: UVec2::new(1, 1),
                };
            });
    }

    //Plot Panel
    if ui_state.show_plots {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(400.0)
            .max_height(700.)
            .show(ctx, |ui| {
                ui.heading("Microphone Plot");

                egui::ComboBox::from_label("Select Plot Type")
                    .selected_text(format!("{:?}", ui_state.plot_type))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.selectable_value(
                            &mut ui_state.plot_type,
                            PlotType::TimeDomain,
                            "Time Domain",
                        );
                        ui.selectable_value(
                            &mut ui_state.plot_type,
                            PlotType::FrequencyDomain,
                            "Frequency Domain",
                        );
                    });

                match ui_state.plot_type {
                    PlotType::TimeDomain => {
                        ui.separator();
                        Plot::new("mic_plot")
                            .allow_zoom([true, false])
                            // .allow_scroll(false)
                            .x_axis_label("Time (s)")
                            .y_axis_label("Amplitude")
                            .legend(Legend::default())
                            .show(ui, |plot_ui| {
                                for mic in microphones.iter() {
                                    //TODO: because of this clone, the app is getting slower as time goes on (because the vec is getting bigger)
                                    let points: PlotPoints = PlotPoints::new(mic.record.clone());
                                    let line = Line::new(points);
                                    plot_ui.line(line.name(format!(
                                        "Microphone {} (x: {}, y: {})",
                                        mic.id, mic.x, mic.y
                                    )));
                                }
                            });
                    }

                    PlotType::FrequencyDomain => {
                        egui::ComboBox::from_label("FFT Microphone")
                            .selected_text(if let Some(index) = ui_state.current_fft_microphone {
                                format!("Microphone {index}")
                            } else {
                                "No Microphone Selected".to_string()
                            })
                            .show_ui(ui, |ui| {
                                for mic in microphones.iter() {
                                    ui.selectable_value(
                                        &mut ui_state.current_fft_microphone,
                                        Some(mic.id),
                                        format!("Microphone {}", mic.id),
                                    );
                                }
                            });
                        ui.separator();
                        Plot::new("mic_plot")
                            .allow_zoom([true, false])
                            .allow_scroll(false)
                            .allow_drag(false)
                            .allow_boxed_zoom(false)
                            .x_axis_label("Frequency (Hz)")
                            .y_axis_label("Intensity")
                            .show(ui, |plot_ui| {
                                if ui_state.current_fft_microphone.is_none() {
                                    return;
                                }

                                let mut mic = microphones
                                    .iter_mut()
                                    .find(|m| {
                                        m.id == ui_state
                                            .current_fft_microphone
                                            .expect("no mic selected")
                                    })
                                    .unwrap();

                                let mapped_spectrum =
                                    calc_mic_spectrum(&mut mic, grid.delta_t, &ui_state);

                                let points = PlotPoints::new(mapped_spectrum);
                                let line = Line::new(points);
                                plot_ui.line(line);
                            });
                    }
                }
            });
    }

    // Tool Panel
    egui::SidePanel::left("tool_panel")
        .frame(
            Frame::default()
                .inner_margin(Margin {
                    left: 8., //looks better
                    right: 10.,
                    top: 10.,
                    bottom: 10.,
                })
                .fill(Color32::from_rgb(25, 25, 25)),
        )
        .default_width(35.)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_enabled(!ui_state.render_abc_area);
            //Tests for tool buttons

            for tool_type in ToolType::TYPES {
                if ui
                    .add(
                        egui::Button::image_and_text(
                            egui::load::SizedTexture::new(cursor_icon, [25., 25.]),
                            "",
                        )
                        .fill(if tool_type == ui_state.current_tool {
                            Color32::DARK_GRAY
                        } else {
                            Color32::TRANSPARENT
                        })
                        .min_size(Vec2::new(0., 35.)),
                    )
                    .on_hover_text(format!("{:?}", tool_type))
                    .clicked()
                {
                    ui_state.current_tool = tool_type;
                }
                ui.add_space(4.);
            }
        });

    // Main Render Area
    egui::CentralPanel::default()
        .frame(
            Frame::default()
                .inner_margin(Margin {
                    left: 20.,
                    right: 20.,
                    top: 20.,
                    bottom: 20.,
                })
                .fill(Color32::from_rgb(25, 25, 25)),
        )
        .show(ctx, |ui| {
            // Main Simulation Area

            let pb = pixel_buffers.iter().next().expect("first pixel buffer");
            let texture = pb.egui_texture();
            // let image = ui.image(egui::load::SizedTexture::new(texture.id, texture.size));

            let image = ui.add(
                egui::Image::new(egui::load::SizedTexture::new(texture.id, texture.size))
                    .shrink_to_fit(),
            );

            ui_state.image_rect = image.rect;

            // Gizmos

            if !ui_state.render_abc_area {
                let painter = ui.painter();

                match ui_state.current_tool {
                    ToolType::MoveSource => {
                        for source in sources.iter() {
                            let gizmo_pos = pos2(
                                f32_map_range(
                                    0.,
                                    SIMULATION_WIDTH as f32,
                                    image.rect.min.x,
                                    image.rect.max.x,
                                    source.x as f32,
                                ),
                                f32_map_range(
                                    0.,
                                    SIMULATION_HEIGHT as f32,
                                    image.rect.min.y,
                                    image.rect.max.y,
                                    source.y as f32,
                                ),
                            );

                            painter.add(egui::Shape::Circle(CircleShape::stroke(
                                gizmo_pos,
                                10.,
                                Stroke::new(10.0, Color32::from_rgb(255, 100, 0)),
                            )));
                        }
                    }
                    ToolType::MoveWall => {
                        for wall in wallblocks.iter() {
                            let gizmo_pos = pos2(
                                f32_map_range(
                                    0.,
                                    SIMULATION_WIDTH as f32,
                                    image.rect.min.x,
                                    image.rect.max.x,
                                    wall.rect.center().x,
                                ),
                                f32_map_range(
                                    0.,
                                    SIMULATION_HEIGHT as f32,
                                    image.rect.min.y,
                                    image.rect.max.y,
                                    wall.rect.center().y,
                                ),
                            );

                            painter.add(egui::Shape::Circle(CircleShape::stroke(
                                gizmo_pos,
                                10.,
                                Stroke::new(10.0, Color32::from_rgb(255, 100, 0)),
                            )));
                        }
                    }
                    ToolType::ResizeWall => {
                        for wall in wallblocks.iter() {
                            let gizmo_pos = pos2(
                                f32_map_range(
                                    0.,
                                    SIMULATION_WIDTH as f32,
                                    image.rect.min.x,
                                    image.rect.max.x,
                                    wall.rect.max.x,
                                ),
                                f32_map_range(
                                    0.,
                                    SIMULATION_HEIGHT as f32,
                                    image.rect.min.y,
                                    image.rect.max.y,
                                    wall.rect.max.y,
                                ),
                            );

                            painter.add(egui::Shape::Circle(CircleShape::filled(
                                gizmo_pos,
                                5.,
                                Color32::from_rgb(54, 188, 255),
                            )));
                        }
                    }
                    _ => {}
                }
            }
        });
}
