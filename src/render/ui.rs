use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::epaint::CircleShape;
use bevy_pixel_buffer::bevy_egui::egui::{pos2, Color32, Frame, Margin, Stroke, TextureOptions};
use bevy_pixel_buffer::bevy_egui::{egui, EguiContexts};
use bevy_pixel_buffer::prelude::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::components::microphone::*;
use crate::components::source::*;
use crate::grid::grid::Grid;
use crate::math::constants::*;
use crate::math::fft::calc_mic_spectrum;
use crate::math::transformations::u32_map_range;
use crate::render::state::*;

pub fn draw_egui(
    mut pixel_buffers: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    mut sources: Query<&mut Source>,
    mut microphones: Query<&mut Microphone>,
    mut ui_state: ResMut<UiState>,
    diagnostics: Res<DiagnosticsStore>,
    grid: Res<Grid>,
    images: Local<Images>,
) {
    let cursor_icon = egui_context.add_image(images.cursor_icon.clone_weak());

    let ctx = egui_context.ctx_mut();
    egui::SidePanel::left("left_panel")
        .default_width(300.)
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
                                });
                        });
                    }
                });

            ui.separator();

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

            egui::TopBottomPanel::bottom("general_settings_bottom_panel").show_inside(ui, |ui| {
                ui.heading("General Settings");
                ui.separator();

                if ui
                    .button(if ui_state.is_running { "Stop" } else { "Start" })
                    .clicked()
                {
                    ui_state.is_running = !ui_state.is_running;
                }

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
                        for (index, mut pb) in pixel_buffers.iter_mut().enumerate() {
                            if index == 0 {
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
                        }
                    }

                    if ui
                        .checkbox(&mut ui_state.render_abc_area, "Render Absorbing Boundary")
                        .clicked()
                    {
                        for (index, mut pb) in pixel_buffers.iter_mut().enumerate() {
                            if index == 0 {
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
                        }
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

                ui.checkbox(&mut ui_state.show_fft, "Show FFT");

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

                if ui
                    .checkbox(&mut ui_state.show_plots, "Show Plots")
                    .clicked()
                    && !ui_state.show_fft
                {
                    for mut mic in microphones.iter_mut() {
                        mic.clear();
                    }
                }

                ui.add_space(10.);
            })
        });

    egui::CentralPanel::default()
        .frame(
            Frame::default()
                .inner_margin(Margin {
                    left: 0.,
                    right: 0.,
                    top: 0.,
                    bottom: 0.,
                })
                .fill(Color32::from_rgb(25, 25, 25)),
        )
        .show(ctx, |ui| {
            // Tool Panel

            egui::SidePanel::left("tool_panel")
                .frame(
                    Frame::default()
                        .inner_margin(Margin {
                            left: 0.,
                            right: 0.,
                            top: 0.,
                            bottom: 0.,
                        })
                        .fill(Color32::from_rgb(25, 25, 25)),
                )
                .default_width(35.)
                .resizable(false)
                .show_inside(ui, |ui| {
                    //Tests for tool buttons
                    ui.add_space(4.);

                    ui.add(
                        egui::Image::new(egui::load::SizedTexture::new(cursor_icon, [25., 25.]))
                            .bg_fill(Color32::DARK_GRAY)
                            .shrink_to_fit(),
                    );

                    ui.add_space(2.);

                    ui.add(
                        egui::Image::new(egui::load::SizedTexture::new(cursor_icon, [25., 25.]))
                            .shrink_to_fit(),
                    );

                    ui.add_space(2.);

                    ui.add(
                        egui::Image::new(egui::load::SizedTexture::new(cursor_icon, [25., 25.]))
                            .shrink_to_fit(),
                    );

                    ui.add_space(2.);

                    ui.vertical(|ui| {
                        ui.selectable_value(
                            &mut ui_state.tool_type,
                            ToolType::PlaceSource,
                            "Place",
                        );
                        ui.selectable_value(&mut ui_state.tool_type, ToolType::MoveSource, "Move");
                    });
                });

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

            if image.hovered() && ui_state.tool_type == ToolType::MoveSource {
                let painter = ui.painter();

                for source in sources.iter() {
                    let gizmo_pos = pos2(
                        u32_map_range(
                            0,
                            SIMULATION_WIDTH,
                            image.rect.min.x as u32,
                            image.rect.max.x as u32,
                            source.x,
                        ) as f32,
                        u32_map_range(
                            0,
                            SIMULATION_HEIGHT,
                            image.rect.min.y as u32,
                            image.rect.max.y as u32,
                            source.y,
                        ) as f32,
                    );

                    painter.add(egui::Shape::Circle(CircleShape::stroke(
                        gizmo_pos,
                        10.,
                        Stroke::new(10.0, Color32::from_rgb(255, 100, 0)),
                    )));
                }
            }
        });

    if ui_state.show_fft {
        egui::SidePanel::right("spectrum_panel")
            .default_width(400.)
            // .resizable(false)
            .show(ctx, |ui| {
                let pb = pixel_buffers.iter().nth(1).expect("second pixel buffer");
                let texture = pb.egui_texture();

                ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(texture.id, texture.size))
                        .shrink_to_fit(),
                );
            });
    }

    if ui_state.show_plots {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(400.0)
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

                ui.separator();

                match ui_state.plot_type {
                    PlotType::TimeDomain => {
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

                                let mapped_spectrum = calc_mic_spectrum(&mut mic, grid.delta_t);

                                let points = PlotPoints::new(mapped_spectrum);
                                let line = Line::new(points);
                                plot_ui.line(line);
                            });
                    }
                }
            });
    }
}
