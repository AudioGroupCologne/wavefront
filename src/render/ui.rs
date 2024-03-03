use std::ffi::OsStr;
use std::path::Path;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::epaint::CircleShape;
use bevy_pixel_buffer::bevy_egui::egui::{pos2, Color32, Frame, Margin, Vec2};
use bevy_pixel_buffer::bevy_egui::EguiContexts;
use bevy_pixel_buffer::prelude::*;
use egui_file::FileDialog;
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::components::microphone::*;
use crate::components::source::*;
use crate::components::states::{Gizmo, MenuSelected, Overlay, Selected};
use crate::components::wall::WallBlock;
use crate::grid::grid::Grid;
use crate::math::constants::*;
use crate::math::fft::calc_mic_spectrum;
use crate::math::transformations::f32_map_range;
use crate::render::state::*;

pub fn draw_egui(
    mut commands: Commands,
    diagnostics: Res<DiagnosticsStore>,
    images: Local<Images>,
    mut pixel_buffers: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut grid: ResMut<Grid>,
    mut wallblock_set: ParamSet<(
        Query<(Entity, &mut WallBlock), Without<Overlay>>,
        Query<(Entity, &mut WallBlock), (Without<Overlay>, With<Selected>)>,
        Query<(Entity, &mut WallBlock), (Without<Overlay>, With<MenuSelected>)>,
        Query<&WallBlock>,
    )>,
    mut source_set: ParamSet<(
        Query<(Entity, &mut Source)>,
        Query<(Entity, &Source), With<Selected>>,
        Query<(Entity, &Source), With<MenuSelected>>,
        Query<&Source>,
    )>,
    mut mic_set: ParamSet<(
        Query<(Entity, &mut Microphone)>,
        Query<(Entity, &Microphone), With<Selected>>,
        Query<(Entity, &Microphone), With<MenuSelected>>,
        Query<&Microphone>,
    )>,
) {
    //Icons
    let _cursor_icon = egui_context.add_image(images.cursor_icon.clone_weak());

    let icon_vec = vec![
        (
            ToolType::PlaceSource,
            egui_context.add_image(images.place_source_icon.clone_weak()),
        ),
        (
            ToolType::MoveSource,
            egui_context.add_image(images.move_source_icon.clone_weak()),
        ),
        (
            ToolType::DrawWall,
            egui_context.add_image(images.draw_wall_icon.clone_weak()),
        ),
        (
            ToolType::ResizeWall,
            egui_context.add_image(images.resize_wall_icon.clone_weak()),
        ),
        (
            ToolType::MoveWall,
            egui_context.add_image(images.move_wall_icon.clone_weak()),
        ),
        (
            ToolType::PlaceMic,
            egui_context.add_image(images.place_mic_icon.clone_weak()),
        ),
        (
            ToolType::MoveMic,
            egui_context.add_image(images.move_mic_icon.clone_weak()),
        ),
    ];

    let ctx = egui_context.ctx_mut();

    // Side Panel (Sources, Mic, Walls, Tool Options, Settings)
    egui::SidePanel::left("left_panel")
        .default_width(450.)
        .show(ctx, |ui| {
            // not a perfect solution -> when resizing this will set tools_enabled to true
            ui_state.tools_enabled = !ui.rect_contains_pointer(ui.available_rect_before_wrap())
                && !ui_state.render_abc_area;

            ui.spacing_mut().slider_width = 200.0;

            ui.heading("Settings");
            if let Some(value) = diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.smoothed())
            {
                ui.label(format!("FPS: {:.1}", value));
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .button("save")
                    .on_hover_text("Save the current state of the simulation")
                    .clicked()
                {
                    // TODO: force saving as .json?
                    let mut dialog = FileDialog::save_file(None);
                    dialog.open();
                    ui_state.save_file_dialog = Some(dialog);
                }
                if let Some(dialog) = &mut ui_state.save_file_dialog {
                    if dialog.show(ctx).selected() {
                        if let Some(path) = dialog.path() {
                            let source_set = source_set.p3();
                            let mic_set = mic_set.p3();
                            let wallblock_set = wallblock_set.p3();

                            let sources = source_set.iter().collect::<Vec<_>>();
                            let mics = mic_set.iter().collect::<Vec<_>>();
                            let wallblocks = wallblock_set.iter().collect::<Vec<_>>();

                            crate::saving::save(path, &sources, &mics, &wallblocks).unwrap();
                        }
                    }
                }

                if ui
                    .button("load")
                    .on_hover_text("Load a previously saved state of the simulation")
                    .clicked()
                {
                    let filter = Box::new({
                        let ext = Some(OsStr::new("json"));
                        move |path: &Path| -> bool { path.extension() == ext }
                    });
                    let mut dialog = FileDialog::open_file(None).show_files_filter(filter);
                    dialog.open();
                    ui_state.open_file_dialog = Some(dialog);
                }

                if let Some(dialog) = &mut ui_state.open_file_dialog {
                    if dialog.show(ctx).selected() {
                        if let Some(path) = dialog.path() {
                            let save_data = crate::loading::load(path);

                            // Clear all entities
                            for (entity, _) in source_set.p0().iter() {
                                commands.entity(entity).despawn();
                            }
                            for (entity, _) in mic_set.p0().iter() {
                                commands.entity(entity).despawn();
                            }
                            for (entity, _) in wallblock_set.p0().iter() {
                                commands.entity(entity).despawn();
                            }

                            // Load entities
                            for source in save_data.sources {
                                commands.spawn(source);
                            }
                            for mic in save_data.mics {
                                commands.spawn(mic);
                            }
                            for wallblock in save_data.wallblocks {
                                commands.spawn(wallblock);
                            }
                        }
                    }
                }
            });

            // Sources
            egui::ScrollArea::vertical()
                .id_source("source_scroll_area")
                .max_height(400.)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    let mut binding = source_set.p0();
                    let mut source_vec = binding.iter_mut().collect::<Vec<_>>();
                    source_vec.sort_by_cached_key(|(_, source)| source.id);

                    source_vec.iter_mut().for_each(|(entity, ref mut source)| {
                        let collapse = ui.collapsing(format!("Source {}", source.id), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("x:");
                                ui.add(
                                    egui::DragValue::new(&mut source.x)
                                        .speed(1)
                                        .clamp_range(0.0..=SIMULATION_WIDTH as f32 - 1.),
                                );
                                ui.add_space(10.);
                                ui.label("y:");
                                ui.add(
                                    egui::DragValue::new(&mut source.y)
                                        .speed(1)
                                        .clamp_range(0.0..=SIMULATION_HEIGHT as f32 - 1.),
                                );
                            });
                            ui.add(
                                egui::Slider::new(&mut source.frequency, 0.0..=20000.0)
                                    .text("Frequency (Hz)"),
                            );
                            ui.add(
                                egui::Slider::new(&mut source.amplitude, 0.0..=25.0)
                                    .text("Amplitude"),
                            );
                            ui.add(
                                egui::Slider::new(&mut source.phase, 0.0..=360.0).text("Phase (°)"),
                            );
                            egui::ComboBox::from_label("Waveform")
                                .selected_text(format!("{:?}", source.source_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut source.source_type,
                                        SourceType::Sin,
                                        "Sinus",
                                    );
                                    ui.selectable_value(
                                        &mut source.source_type,
                                        SourceType::Gauss,
                                        "Gauss",
                                    );
                                    ui.selectable_value(
                                        &mut source.source_type,
                                        SourceType::WhiteNoise,
                                        "White Noise",
                                    );
                                });
                            if ui.add(egui::Button::new("Delete")).clicked() {
                                commands.entity(*entity).despawn();
                            }
                        });
                        if collapse.header_response.clicked() {
                            if collapse.openness < 0.5 {
                                commands.entity(*entity).insert(MenuSelected);
                            } else {
                                commands.entity(*entity).remove::<MenuSelected>();
                            }
                        }
                    });
                });

            ui.separator();

            // Microphones
            egui::ScrollArea::vertical()
                .id_source("mic_scroll_area")
                .max_height(400.)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    let mut binding = mic_set.p0();
                    let mut mic_vec = binding.iter_mut().collect::<Vec<_>>();
                    mic_vec.sort_by_cached_key(|(_, mic)| mic.id);

                    mic_vec.iter_mut().for_each(|(entity, ref mut mic)| {
                        let collapse = ui.collapsing(format!("Microphone {}", mic.id), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("x:");
                                ui.add(
                                    egui::DragValue::new(&mut mic.x)
                                        .speed(1)
                                        .clamp_range(0.0..=SIMULATION_WIDTH as f32 - 1.),
                                );
                                ui.add_space(10.);
                                ui.label("y:");
                                ui.add(
                                    egui::DragValue::new(&mut mic.y)
                                        .speed(1)
                                        .clamp_range(0.0..=SIMULATION_HEIGHT as f32 - 1.),
                                );
                            });
                            if ui.add(egui::Button::new("Delete")).clicked() {
                                commands.entity(*entity).despawn();
                            }
                        });
                        if collapse.header_response.clicked() {
                            if collapse.openness < 0.5 {
                                commands.entity(*entity).insert(MenuSelected);
                            } else {
                                commands.entity(*entity).remove::<MenuSelected>();
                            }
                        }
                    });
                });

            ui.separator();

            // Walls
            egui::ScrollArea::vertical()
                .id_source("wallblock_scroll_area")
                .max_height(400.)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    let mut binding = wallblock_set.p0();
                    let mut wb_vec = binding.iter_mut().collect::<Vec<_>>();
                    wb_vec.sort_by_cached_key(|(_, wb)| wb.id);

                    wb_vec.iter_mut().for_each(|(entity, ref mut wb)| {
                        let collapse = ui.collapsing(format!("Wallblock {}", wb.id), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Top Corner x:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wb.rect.min.x)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_WIDTH as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    wb.update_calc_rect(ui_state.e_al);
                                }
                                ui.add_space(10.);
                                ui.label("Top Corner x:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wb.rect.min.y)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_WIDTH as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    wb.update_calc_rect(ui_state.e_al);
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Bottom Corner x:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wb.rect.max.x)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_WIDTH as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    wb.update_calc_rect(ui_state.e_al);
                                }
                                ui.add_space(10.);
                                ui.label("Bottom Corner y:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wb.rect.max.y)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_HEIGHT as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    wb.update_calc_rect(ui_state.e_al);
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "Width: {:.3} m",
                                    (wb.rect.width() + 1.0) * ui_state.delta_l
                                ));

                                ui.add_space(10.);

                                ui.label(format!(
                                    "Height: {:.3} m",
                                    (wb.rect.height() + 1.0) * ui_state.delta_l
                                ));
                            });

                            if ui.add(egui::Button::new("Delete")).clicked() {
                                commands.entity(*entity).despawn();
                            }
                        });
                        if collapse.header_response.clicked() {
                            if collapse.openness < 0.5 {
                                commands.entity(*entity).insert(MenuSelected);
                            } else {
                                commands.entity(*entity).remove::<MenuSelected>();
                            }
                        }
                    });
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
                    for (_, mut mic) in mic_set.p0().iter_mut() {
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
                    for (_, mut mic) in mic_set.p0().iter_mut() {
                        mic.clear();
                    }
                }

                // ABC
                if ui
                    .checkbox(&mut ui_state.render_abc_area, "Render Absorbing Boundary")
                    .clicked()
                {
                    ui_state.tools_enabled = !ui_state.render_abc_area;
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
                ui.collapsing("ABC", |ui| {
                    ui.set_enabled(ui_state.render_abc_area);
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

                        for (_, mut wb) in wallblock_set.p0().iter_mut() {
                            wb.update_calc_rect(ui_state.e_al);
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
                    ToolType::MoveMic => {}
                    ToolType::PlaceMic => {}
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
            // .resizable(false)
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
                                for (_, mic) in mic_set.p0().iter() {
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
                                for (_, mic) in mic_set.p0().iter() {
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

                                // this is not my doing, but if it works, it works
                                let mut binding = mic_set.p0();
                                let mut mic = binding
                                    .iter_mut()
                                    .find(|m| {
                                        m.1.id
                                            == ui_state
                                                .current_fft_microphone
                                                .expect("no mic selected")
                                    })
                                    .unwrap()
                                    .1;

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
            ui.set_enabled(ui_state.tools_enabled);

            for (tool_type, tool_icon) in icon_vec {
                if ui
                    .add(
                        egui::Button::image_and_text(
                            egui::load::SizedTexture::new(tool_icon, [25., 25.]),
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
            ui.set_min_width(100.);
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
                //menu gizmos
                if !ui_state.tools_enabled {
                    for (_, mic) in mic_set.p2().iter() {
                        let gizmo_pos = pos2(
                            f32_map_range(
                                0.,
                                SIMULATION_WIDTH as f32,
                                image.rect.min.x,
                                image.rect.max.x,
                                mic.x as f32,
                            ),
                            f32_map_range(
                                0.,
                                SIMULATION_HEIGHT as f32,
                                image.rect.min.y,
                                image.rect.max.y,
                                mic.y as f32,
                            ),
                        );

                        painter.add(egui::Shape::Circle(CircleShape::filled(
                            gizmo_pos,
                            10.,
                            Color32::from_rgb(0, 0, 255),
                        )));
                    }
                    for (_, source) in source_set.p2().iter() {
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

                        painter.add(egui::Shape::Circle(CircleShape::filled(
                            gizmo_pos,
                            10.,
                            Color32::from_rgb(0, 255, 0),
                        )));
                    }
                    for (_, wall) in wallblock_set.p2().iter() {
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

                        painter.add(egui::Shape::Circle(CircleShape::filled(
                            gizmo_pos,
                            10.,
                            Color32::from_rgb(255, 0, 0),
                        )));
                    }
                } else {
                    // Tool specific gizmos
                    match ui_state.current_tool {
                        ToolType::MoveSource => {
                            for (_, source) in source_set.p0().iter() {
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

                                painter.add(egui::Shape::Circle(CircleShape::filled(
                                    gizmo_pos,
                                    5.,
                                    Color32::from_rgb(255, 100, 0),
                                )));
                            }
                            for (_, source) in source_set.p1().iter() {
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

                                painter.add(egui::Shape::Circle(CircleShape::filled(
                                    gizmo_pos,
                                    10.,
                                    Color32::from_rgb(255, 120, 50),
                                )));
                            }
                        }
                        ToolType::MoveWall => {
                            for (_, wall) in wallblock_set.p0().iter() {
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

                                painter.add(egui::Shape::Circle(CircleShape::filled(
                                    gizmo_pos,
                                    5.,
                                    Color32::from_rgb(255, 100, 0),
                                )));
                            }
                            for (_, wall) in wallblock_set.p1().iter() {
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

                                painter.add(egui::Shape::Circle(CircleShape::filled(
                                    gizmo_pos,
                                    10.,
                                    Color32::from_rgb(255, 100, 0),
                                )));
                            }
                        }
                        ToolType::ResizeWall => {
                            for (_, wall) in wallblock_set.p0().iter() {
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
                        ToolType::PlaceSource => {
                            for (_, source) in source_set.p0().iter() {
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

                                painter.add(egui::Shape::Circle(CircleShape::filled(
                                    gizmo_pos,
                                    2.5,
                                    Color32::from_rgb(255, 100, 0),
                                )));
                            }
                        }
                        ToolType::DrawWall => {}
                        ToolType::PlaceMic => {
                            for (_, mic) in mic_set.p0().iter() {
                                let gizmo_pos = mic.get_gizmo_position(&image.rect);

                                painter.add(egui::Shape::Circle(CircleShape::filled(
                                    gizmo_pos,
                                    2.5,
                                    Color32::from_rgb(0, 100, 255),
                                )));
                            }
                        }
                        ToolType::MoveMic => {
                            for (_, mic) in mic_set.p0().iter() {
                                let gizmo_pos = mic.get_gizmo_position(&image.rect);

                                painter.add(egui::Shape::Circle(CircleShape::filled(
                                    gizmo_pos,
                                    5.,
                                    Color32::from_rgb(0, 100, 255),
                                )));
                            }
                            // selected mics
                            for (_, mic) in mic_set.p1().iter() {
                                let gizmo_pos = mic.get_gizmo_position(&image.rect);

                                painter.add(egui::Shape::Circle(CircleShape::filled(
                                    gizmo_pos,
                                    10.,
                                    Color32::from_rgb(100, 150, 255),
                                )));
                            }
                        }
                    }
                }
            }
        });
}
