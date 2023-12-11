use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::epaint::CircleShape;
use bevy_pixel_buffer::bevy_egui::egui::{pos2, Color32, Pos2, Rect, Rounding, Stroke};
use bevy_pixel_buffer::bevy_egui::{egui, EguiContexts};
use bevy_pixel_buffer::prelude::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use spectrum_analyzer::scaling::scale_to_zero_to_one;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

use crate::components::{u32_map_range, GradientResource, Microphone, Source, SourceType};
use crate::constants::*;
use crate::grid::Grid;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AttenuationType {
    Power,
    OriginalOneWay,
    Linear,
    Old,
    DoNothing,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PlotType {
    TimeDomain,
    FrequencyDomain,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ToolType {
    PlaceSource,
    MoveSource,
}

#[derive(Resource)]
pub struct UiState {
    pub is_running: bool,
    pub delta_l: f32,
    pub epsilon: f32,
    pub e_al: u32,
    pub render_abc_area: bool,
    pub at_type: AttenuationType,
    pub power_order: u32,
    pub image_rect: egui::emath::Rect,
    pub show_fft: bool,
    pub show_mic_plot: bool,
    pub current_fft_microphone: Option<usize>,
    pub plot_type: PlotType,
    pub tool_type: ToolType,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            is_running: true,
            delta_l: 0.001,
            epsilon: 0.001,
            e_al: 50,
            render_abc_area: false,
            at_type: AttenuationType::Power,
            power_order: 5,
            image_rect: egui::emath::Rect::NOTHING,
            show_fft: false,
            show_mic_plot: false,
            current_fft_microphone: None,
            plot_type: PlotType::TimeDomain,
            tool_type: ToolType::PlaceSource,
        }
    }
}

pub struct Images {
    cursor_icon: Handle<Image>,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            cursor_icon: asset_server.load("test_icon.png"),
        }
    }
}

const CHUNK_SIZE: usize = 2usize.pow(11); // 2048

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
                                .selected_text(format!("{:?}", s.r#type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut s.r#type, SourceType::Sin, "Sinus");
                                    ui.selectable_value(&mut s.r#type, SourceType::Gauss, "Gauss");
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

                if ui
                    .checkbox(&mut ui_state.show_mic_plot, "Show Microphone Plot")
                    .clicked()
                {
                    if !ui_state.show_fft {
                        for mut mic in microphones.iter_mut() {
                            mic.clear();
                        }
                    }
                }

                ui.add_space(10.);
            })
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        // Tool Panel

        egui::SidePanel::left("tool_panel")
            .default_width(50.)
            .resizable(false)
            .show_inside(ui, |ui| {
                //Tests for tool buttons
                ui.add(
                    egui::ImageButton::new(egui::load::SizedTexture::new(cursor_icon, [25., 25.]))
                        .selected(true),
                );

                ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(cursor_icon, [25., 25.]))
                        .bg_fill(Color32::DARK_GRAY)
                        .shrink_to_fit(),
                );

                ui.vertical(|ui| {
                    ui.selectable_value(&mut ui_state.tool_type, ToolType::PlaceSource, "Place");
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
                Plot::new("mic_plot")
                    .allow_zoom([true, false])
                    .allow_scroll(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .x_axis_label("Frequency (Hz)")
                    .y_axis_label("Intensity")
                    // .x_grid_spacer(log_grid_spacer(10)) // doesn't do anything
                    .view_aspect(1.5)
                    .show(ui, |plot_ui| {
                        if ui_state.current_fft_microphone.is_none() {
                            return;
                        }
                        let mut mic = microphones
                            .iter_mut()
                            .find(|m| {
                                m.id == ui_state.current_fft_microphone.expect("no mic selected")
                            })
                            .unwrap();

                        let samples = if mic.record.len() < CHUNK_SIZE {
                            let mut s = mic.record.clone();
                            s.resize(CHUNK_SIZE, [0.0, 0.0]);
                            s
                        } else {
                            mic.record[mic.record.len() - CHUNK_SIZE..].to_vec()
                        };

                        let hann_window =
                            hann_window(&samples.iter().map(|x| x[1] as f32).collect::<Vec<_>>());

                        let spectrum_hann_window = samples_fft_to_spectrum(
                            &hann_window,
                            (1. / grid.delta_t) as u32,
                            FrequencyLimit::All,
                            Some(&scale_to_zero_to_one),
                        )
                        .unwrap();

                        let mapped_spectrum = spectrum_hann_window
                            .data()
                            .iter()
                            // .map(|x| [x.0.val().log10() as f64, x.1.val() as f64])
                            .map(|(x, y)| [x.val() as f64, y.val() as f64])
                            .collect::<Vec<_>>();

                        mic.spektrum.push(mapped_spectrum.clone());
                        if mic.spektrum.len() > 500 {
                            //TODO: hardcode
                            mic.spektrum.remove(0);
                        }

                        let points = PlotPoints::new(mapped_spectrum);
                        let line = Line::new(points);
                        plot_ui.line(line);
                    });

                egui::ComboBox::from_label("Microphone")
                    .selected_text(format!(
                        "{:?}",
                        if let Some(index) = ui_state.current_fft_microphone {
                            format!("Microphone {index}")
                        } else {
                            "No Microphone Selected".to_string()
                        }
                    ))
                    .show_ui(ui, |ui| {
                        for mic in microphones.iter() {
                            ui.selectable_value(
                                &mut ui_state.current_fft_microphone,
                                Some(mic.id),
                                format!("Microphone {}", mic.id),
                            );
                        }
                    });

                let pb = pixel_buffers.iter().nth(1).expect("second pixel buffer");
                let texture = pb.egui_texture();

                ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(texture.id, texture.size))
                        .shrink_to_fit(),
                );
            });
    }

    if ui_state.show_mic_plot {
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

                    PlotType::FrequencyDomain => {}
                }
            });
    }
}

pub fn draw_pixels(
    pixel_buffers: QueryPixelBuffer,
    grid: Res<Grid>,
    gradient: Res<GradientResource>,
    ui_state: Res<UiState>,
    microphones: Query<&Microphone>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut items = query.iter();
    let mut frame = images.frame(items.next().expect("one pixel buffer"));

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

    let mut frame = images.frame(items.next().expect("two pixel buffers"));
    if ui_state.show_fft && ui_state.current_fft_microphone.is_some() {
        //? for now we don't draw the spectrum if no mic is selected, is this ok?
        // paint spectrum
        let mic = microphones
            .iter()
            .find(|m| m.id == ui_state.current_fft_microphone.expect("no mic selected"))
            .unwrap();
        let spectrum = &mic.spektrum;
        let len_y = spectrum.len();

        // if len_y > 0 {
        //     println!("{:?}", spectrum);
        // }

        frame.per_pixel_par(|coords, _| Pixel {
            r: (if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize][u32_map_range(0, 250, 0, 120, coords.x) as usize][1]
                    * 255.
            //TODO: is 120 hardcoded + 250 is hardcoded
            } else {
                0.
            }) as u8,
            g: (if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize][u32_map_range(0, 250, 0, 120, coords.x) as usize][1]
                    * 255.
            //TODO: is 120 hardcoded + 250 is hardcoded
            } else {
                0.
            }) as u8,
            b: (if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize][u32_map_range(0, 250, 0, 120, coords.x) as usize][1]
                    * 255.
            //TODO: is 120 hardcoded + 250 is hardcoded
            } else {
                0.
            }) as u8,
            a: 255,
        });
    }
}

// pub fn draw_walls(mut pb: QueryPixelBuffer, walls: Query<&Wall>, ui_state: Res<UiState>) {
//     let mut frame = pb.frame();
//     for wall in walls.iter() {
//         let (x, y) = Grid::index_to_coords(wall.0 as u32, ui_state.e_al);
//         frame
//             .set(
//                 UVec2::new(x, y),
//                 Pixel {
//                     r: 255,
//                     g: 255,
//                     b: 255,
//                     a: 255,
//                 },
//             )
//             .expect("Wall pixel out of bounds");
//     }
// }

pub fn setup_buffers(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let main_size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };
    let spectrum_size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(250, 500), //TODO: hardcode
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };
    insert_pixel_buffer(&mut commands, &mut images, main_size); //main
    insert_pixel_buffer(&mut commands, &mut images, spectrum_size); //spectrum
}

fn insert_pixel_buffer(commands: &mut Commands, images: &mut Assets<Image>, size: PixelBufferSize) {
    PixelBufferBuilder::new()
        .with_render(false)
        .with_size(size)
        .spawn(commands, images);
}
