use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_file_dialog::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::epaint::CircleShape;
use bevy_pixel_buffer::bevy_egui::egui::{pos2, Color32, Frame, Margin, Vec2};
use bevy_pixel_buffer::bevy_egui::EguiContexts;
use bevy_pixel_buffer::prelude::*;
use egui_plot::{GridMark, Legend, Line, Plot, PlotPoints};

use super::dialog::SaveFileContents;
use crate::components::microphone::*;
use crate::components::source::*;
use crate::components::states::{Gizmo, MenuSelected, Overlay, Selected};
use crate::components::wall::{CircWall, RectWall, WResize, Wall};
use crate::grid::grid::Grid;
use crate::math::constants::*;
use crate::math::fft::calc_mic_spectrum;
use crate::math::transformations::{coords_to_index, f32_map_range};
use crate::render::draw::GradientResource;
use crate::ui::state::*;

pub fn draw_egui(
    mut commands: Commands,
    diagnostics: Res<DiagnosticsStore>,
    images: Local<Images>,
    mut pixel_buffers: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut grid: ResMut<Grid>,
    mut gradient: ResMut<GradientResource>,
    mut rect_wall_set: ParamSet<(
        Query<(Entity, &mut RectWall)>,
        Query<(Entity, &mut RectWall), (Without<Overlay>, With<Selected>)>,
        Query<(Entity, &mut RectWall), (Without<Overlay>, With<MenuSelected>)>,
        Query<&RectWall>,
    )>,
    mut circ_wall_set: ParamSet<(
        Query<(Entity, &mut CircWall), Without<Overlay>>,
        Query<(Entity, &mut CircWall), (Without<Overlay>, With<Selected>)>,
        Query<(Entity, &mut CircWall), (Without<Overlay>, With<MenuSelected>)>,
        Query<&CircWall>,
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

            ui.add_space(5.);
            egui::Grid::new("header_grid")
                .min_col_width(450. / 2.)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Settings");
                        if let Some(value) = diagnostics
                            .get(&FrameTimeDiagnosticsPlugin::FPS)
                            .and_then(|fps| fps.smoothed())
                        {
                            ui.label(format!("FPS: {:.1}", value));
                        }
                    });
                    ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .button("Save")
                                .on_hover_text("Save the current state of the simulation")
                                .clicked()
                            {
                                // TODO: not super happy with this, would like to move it to the dialog system
                                let source_set = source_set.p3();
                                let mic_set = mic_set.p3();
                                let rect_wall_set = rect_wall_set.p3();
                                let circ_wall_set = circ_wall_set.p3();

                                let sources = source_set.iter().collect::<Vec<_>>();
                                let mics = mic_set.iter().collect::<Vec<_>>();
                                let rect_walls = rect_wall_set.iter().collect::<Vec<_>>();
                                let circ_walls = circ_wall_set.iter().collect::<Vec<_>>();

                                let data = crate::ui::saving::save(
                                    &sources,
                                    &mics,
                                    &rect_walls,
                                    &circ_walls,
                                )
                                .unwrap();

                                commands
                                    .dialog()
                                    .add_filter("Json", &["json"])
                                    .set_file_name("save.json")
                                    .set_directory("./")
                                    .set_title("Select a file to save to")
                                    .save_file::<SaveFileContents>(data);
                            }

                            if ui
                                .button("Load")
                                .on_hover_text("Load a previously saved state of the simulation")
                                .clicked()
                            {
                                commands
                                    .dialog()
                                    .add_filter("Json", &["json"])
                                    .set_directory("./")
                                    .set_title("Select a file to load")
                                    .load_file::<SaveFileContents>();
                            }
                        });
                        if ui
                            .button("Screenshot")
                            .on_hover_text("Save a screenshot of the simulation")
                            .clicked()
                        {
                            let mut pixels: Vec<[u8; 3]> = Vec::new();

                            for y in ui_state.boundary_width
                                ..(SIMULATION_WIDTH + ui_state.boundary_width)
                            {
                                for x in ui_state.boundary_width
                                    ..(SIMULATION_HEIGHT + ui_state.boundary_width)
                                {
                                    let pressure = grid.pressure
                                        [coords_to_index(x, y, ui_state.boundary_width)];

                                    let color = gradient.at(pressure);

                                    pixels.push([color.r(), color.g(), color.b()]);
                                }
                            }

                            let data = lodepng::encode_memory(
                                &pixels,
                                SIMULATION_WIDTH as usize,
                                SIMULATION_HEIGHT as usize,
                                lodepng::ColorType::RGB,
                                8,
                            )
                            .expect("");

                            commands
                                .dialog()
                                .add_filter("Jpeg", &["jpeg"])
                                .set_file_name("save.jpg")
                                .set_directory("./")
                                .set_title("Select a file to save to")
                                .save_file::<SaveFileContents>(data);
                        }
                    })
                });

            ui.separator();

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
                            if source.source_type != SourceType::WhiteNoise {
                                ui.add(
                                    egui::Slider::new(&mut source.frequency, 0.0..=20000.0)
                                        .text("Frequency (Hz)"),
                                );
                            }
                            ui.add(
                                egui::Slider::new(&mut source.amplitude, 0.0..=25.0)
                                    .text("Amplitude"),
                            );
                            if source.source_type == SourceType::Sin {
                                ui.add(
                                    egui::Slider::new(&mut source.phase, 0.0..=360.0)
                                        .text("Phase (°)"),
                                );
                            }

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
                        if collapse.header_response.contains_pointer()
                            || collapse.body_response.is_some()
                        {
                            commands.entity(*entity).try_insert(MenuSelected);
                        } else {
                            commands.entity(*entity).remove::<MenuSelected>();
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
                        if collapse.header_response.contains_pointer()
                            || collapse.body_response.is_some()
                        {
                            commands.entity(*entity).try_insert(MenuSelected);
                        } else {
                            commands.entity(*entity).remove::<MenuSelected>();
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

                    let mut rect_binding = rect_wall_set.p0();
                    let mut wall_vec = rect_binding.iter_mut().collect::<Vec<_>>();
                    wall_vec.sort_by_cached_key(|(_, wall)| wall.id);

                    wall_vec.iter_mut().for_each(|(entity, ref mut wall)| {
                        let collapse = ui.collapsing(format!("Wallblock {}", wall.id), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Top Corner x:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wall.rect.min.x)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_WIDTH as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    if wall.rect.min.x > wall.rect.max.x {
                                        wall.rect.min.x = wall.rect.max.x;
                                    }
                                }
                                ui.add_space(10.);
                                ui.label("Top Corner x:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wall.rect.min.y)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_WIDTH as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    // wall.update_calc_rect(ui_state.boundary_width);
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Bottom Corner x:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wall.rect.max.x)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_WIDTH as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    // wall.update_calc_rect(ui_state.boundary_width);
                                }
                                ui.add_space(10.);
                                ui.label("Bottom Corner y:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wall.rect.max.y)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_HEIGHT as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    // wall.update_calc_rect(ui_state.boundary_width);
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "Width: {:.3} m",
                                    wall.rect.width() as f32 * ui_state.delta_l
                                ));

                                ui.add_space(10.);

                                ui.label(format!(
                                    "Height: {:.3} m",
                                    wall.rect.height() as f32 * ui_state.delta_l
                                ));
                            });

                            ui.add(
                                egui::Slider::new(&mut wall.reflection_factor, 0.0..=1.0)
                                    .text("Wall Reflection Factor"),
                            );

                            ui.checkbox(&mut wall.is_hollow, "Hollow Wall");

                            if ui.add(egui::Button::new("Delete")).clicked() {
                                commands.entity(*entity).despawn();
                            }
                        });

                        if collapse.header_response.contains_pointer()
                            || collapse.body_response.is_some()
                        {
                            commands.entity(*entity).try_insert(MenuSelected);
                        } else {
                            commands.entity(*entity).remove::<MenuSelected>();
                        }
                    });
                });

            // General Settings
            egui::TopBottomPanel::bottom("general_settings_bottom_panel").show_inside(ui, |ui| {
                ui.heading("General Settings");
                ui.separator();

                ui.color_edit_button_srgba(&mut gradient.0);
                ui.color_edit_button_srgba(&mut gradient.1);

                ui.horizontal(|ui| {
                    if ui
                        .button(if ui_state.is_running { "Stop" } else { "Start" })
                        .clicked()
                    {
                        ui_state.is_running = !ui_state.is_running;
                    }

                    if ui.button("Reset").clicked() {
                        grid.reset_cells(ui_state.boundary_width);
                        for (_, mut mic) in mic_set.p0().iter_mut() {
                            mic.clear();
                        }
                    }
                });

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
                                SIMULATION_WIDTH + 2 * ui_state.boundary_width,
                                SIMULATION_HEIGHT + 2 * ui_state.boundary_width,
                            )
                        } else {
                            UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT)
                        },
                        pixel_size: UVec2::new(1, 1),
                    };
                }
                ui.collapsing("ABC", |ui| {
                    ui.set_enabled(ui_state.render_abc_area);
                    if ui
                        .add(
                            egui::Slider::new(&mut ui_state.boundary_width, 2..=200)
                                .text("boundary_width"),
                        )
                        .changed()
                    {
                        grid.reset_cells(ui_state.boundary_width);
                        let mut pb = pixel_buffers.iter_mut().next().expect("one pixel buffer");
                        pb.pixel_buffer.size = PixelBufferSize {
                            size: if ui_state.render_abc_area {
                                UVec2::new(
                                    SIMULATION_WIDTH + 2 * ui_state.boundary_width,
                                    SIMULATION_HEIGHT + 2 * ui_state.boundary_width,
                                )
                            } else {
                                UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT)
                            },
                            pixel_size: UVec2::new(1, 1),
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
                            .selected_text(format!("{:?}", ui_state.wall_type))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.selectable_value(
                                    &mut ui_state.wall_type,
                                    WallType::Rectangle,
                                    "Rectangle",
                                );
                                ui.selectable_value(
                                    &mut ui_state.wall_type,
                                    WallType::Circle,
                                    "Circle",
                                );
                            });
                        ui.add(
                            egui::Slider::new(&mut ui_state.wall_reflection_factor, 0.0..=1.0)
                                .text("Wall Reflection Factor"),
                        );
                        ui.checkbox(&mut ui_state.wall_is_hollow, "Hollow Wall");
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
                        Plot::new("fft_plot")
                            .allow_zoom([false, false])
                            .allow_scroll(false)
                            .allow_drag(false)
                            .allow_boxed_zoom(false)
                            .x_axis_label("Frequency (Hz)")
                            .y_axis_label("Intensity (dB)")
                            .x_grid_spacer(|input| {
                                println!(
                                    "bounds: {:?}, base: {:?}",
                                    input.bounds, input.base_step_size
                                );

                                // all of this is just to get the grid marks to be at 10^x
                                let mut marks = Vec::new();

                                for i in input.bounds.0 as u32..=input.bounds.1 as u32 {
                                    marks.push(GridMark {
                                        value: i as f64,
                                        step_size: 1.,
                                    });
                                    // for j in 1..9 {
                                    //     let value = i as f64 + j as f64;
                                    //     marks.push(GridMark {
                                    //         value: value.log(10.0),
                                    //         step_size: 0.5,
                                    //     });
                                    // }
                                }
                                marks
                            })
                            .x_axis_formatter(|mark, _, _| {
                                format!("{:.0}", 10_f64.powf(mark.value))
                            })
                            .label_formatter(|_, value| {
                                format!(
                                    "Intensity: {:.2} dB\nFrequency: {:.2} Hz",
                                    value.y,
                                    10_f64.powf(value.x)
                                )
                            })
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
                                // remove the first element, because of log it is at x=-inf
                                let mapped_spectrum = &mapped_spectrum[1..];

                                let points = PlotPoints::new(mapped_spectrum.to_vec());
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
                    for (_, wall) in rect_wall_set.p2().iter() {
                        let gizmo_pos = pos2(
                            f32_map_range(
                                0.,
                                SIMULATION_WIDTH as f32,
                                image.rect.min.x,
                                image.rect.max.x,
                                wall.get_center().x as f32,
                            ),
                            f32_map_range(
                                0.,
                                SIMULATION_HEIGHT as f32,
                                image.rect.min.y,
                                image.rect.max.y,
                                wall.get_center().y as f32,
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
                            for (_, wall) in rect_wall_set.p0().iter() {
                                let gizmo_pos = pos2(
                                    f32_map_range(
                                        0.,
                                        SIMULATION_WIDTH as f32,
                                        image.rect.min.x,
                                        image.rect.max.x,
                                        wall.get_center().x as f32,
                                    ),
                                    f32_map_range(
                                        0.,
                                        SIMULATION_HEIGHT as f32,
                                        image.rect.min.y,
                                        image.rect.max.y,
                                        wall.get_center().y as f32,
                                    ),
                                );

                                painter.add(egui::Shape::Circle(CircleShape::filled(
                                    gizmo_pos,
                                    5.,
                                    Color32::from_rgb(255, 100, 0),
                                )));
                            }
                            for (_, wall) in rect_wall_set.p1().iter() {
                                let gizmo_pos = pos2(
                                    f32_map_range(
                                        0.,
                                        SIMULATION_WIDTH as f32,
                                        image.rect.min.x,
                                        image.rect.max.x,
                                        wall.get_center().x as f32,
                                    ),
                                    f32_map_range(
                                        0.,
                                        SIMULATION_HEIGHT as f32,
                                        image.rect.min.y,
                                        image.rect.max.y,
                                        wall.get_center().y as f32,
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
                            for (_, wall) in rect_wall_set.p0().iter() {
                                let gizmo_pos = pos2(
                                    f32_map_range(
                                        0.,
                                        SIMULATION_WIDTH as f32,
                                        image.rect.min.x,
                                        image.rect.max.x,
                                        wall.get_resize_point(WResize::BottomRight).x as f32,
                                    ),
                                    f32_map_range(
                                        0.,
                                        SIMULATION_HEIGHT as f32,
                                        image.rect.min.y,
                                        image.rect.max.y,
                                        wall.get_resize_point(WResize::BottomRight).y as f32,
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
