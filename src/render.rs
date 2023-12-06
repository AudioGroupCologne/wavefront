use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::Pos2;
use bevy_pixel_buffer::bevy_egui::EguiContexts;
use bevy_pixel_buffer::{bevy_egui::egui, prelude::*};
use egui_plot::{log_grid_spacer, GridInput, Legend, Line, Plot, PlotPoints};
use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

use crate::components::{GradientResource, Microphone, Source, SourceType, Wall};
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

#[derive(Resource)]
pub struct UiState {
    pub is_running: bool,
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
            is_running: true,
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

const CHUNK_SIZE: usize = 2usize.pow(11); // 2048

pub fn draw_egui(
    mut pb: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    mut sources: Query<&mut Source>,
    mut microphones: Query<&mut Microphone>,
    mut ui_state: ResMut<UiState>,
    mut grid: ResMut<Grid>,
    time: Res<Time>,
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

                ui.add_space(10.);
            })
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        let texture = pb.egui_texture();
        let image = ui.image(egui::load::SizedTexture::new(texture.id, texture.size));
        ui_state.image_rect_top = image.rect.min;
    });

    egui::SidePanel::right("spectrum_panel")
        .default_width(400.)
        // .resizable(false)
        .show(ctx, |ui| {
            Plot::new("mic_plot")
                .allow_zoom([true, false])
                .allow_scroll(false)
                .allow_drag(false)
                .x_axis_label("Frequency")
                .y_axis_label("Intensity")
                .legend(Legend::default())
                .x_grid_spacer(log_grid_spacer(10))
                .view_aspect(1.5)
                .show(ui, |plot_ui| {
                    let mut mic = microphones.iter_mut().next().unwrap();

                    let mut samples = if mic.record.len() < CHUNK_SIZE {
                        let mut s = mic.record.clone();
                        s.resize(CHUNK_SIZE, [0.0, 0.0]);
                        s
                    } else {
                        mic.record[mic.record.len() - CHUNK_SIZE..].to_vec()
                    };

                    let hann_window =
                        hann_window(&samples.iter().map(|x| x[1] as f32).collect::<Vec<_>>());

                    let spectrum_hann_window = samples_fft_to_spectrum(
                        // (windowed) samples
                        &hann_window,
                        // sampling rate
                        (1. / grid.delta_t) as u32,
                        // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
                        FrequencyLimit::Max(20000f32),
                        // optional scale
                        // Some(&divide_by_N_sqrt),
                        None,
                    )
                    .unwrap();

                    let mapped_spectrum = spectrum_hann_window
                        .data()
                        .iter()
                        // .map(|x| [x.0.val().log10() as f64, x.1.val() as f64])
                        .map(|x| [x.0.val() as f64, x.1.val() as f64])
                        .collect::<Vec<_>>();

                    mic.spektrum.push(mapped_spectrum.clone());

                    let points = PlotPoints::new(mapped_spectrum);
                    // dbg!(points.points());
                    let line = Line::new(points);
                    plot_ui.line(line.name("Spectrum".to_string()));
                });

            let texture = pb.egui_texture();
            ui.image(egui::load::SizedTexture::new(texture.id, texture.size));
        });

    egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .default_height(400.0)
        .show(ctx, |ui| {
            ui.heading("Microphone Plot");
            ui.separator();
            //still need to enable a legend
            Plot::new("mic_plot")
                .allow_zoom([true, false])
                // .allow_scroll(false)
                .x_axis_label("Time")
                .y_axis_label("Amplitude")
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    for mic in microphones.iter() {
                        let points: PlotPoints = PlotPoints::new(mic.record.clone());
                        let line = Line::new(points);
                        plot_ui.line(line.name(format!("Microphone(x: {}, y: {})", mic.x, mic.y)));
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
        frame
            .set(
                UVec2::new(x, y),
                Pixel {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
            )
            .expect("Wall pixel out of bounds");
    }
}

// spawn multiple image buffers from examples
#[derive(Component)]
struct MyBuffer {
    shown: bool,
    id: usize,
}

pub fn setup_buffers(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let main_size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        // size: UVec2::new(SIMULATION_WIDTH + 2 * E_AL, SIMULATION_HEIGHT + 2 * E_AL), // render abc
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };
    insert_pixel_buffer(&mut commands, &mut images, 0, main_size);
}

fn insert_pixel_buffer(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    id: usize,
    size: PixelBufferSize,
) {
    PixelBufferBuilder::new()
        .with_render(false)
        .with_size(size)
        .spawn(commands, images)
        .entity()
        .insert(MyBuffer {
            shown: true,
            id: id,
        });
}
