use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_file_dialog::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::{Color32, Frame, Margin, Vec2};
use bevy_pixel_buffer::bevy_egui::EguiContexts;
use bevy_pixel_buffer::prelude::*;
use egui_extras::{Column, TableBuilder};

use super::loading::SaveFileContents;
use super::tabs::{DockState, PlotTabs};
use crate::components::gizmo::GizmoComponent;
use crate::components::microphone::*;
use crate::components::source::*;
use crate::components::states::{MenuSelected, Selected};
use crate::components::wall::{CircWall, RectWall, WResize};
use crate::events::{Load, Reset, Save, UpdateWalls};
use crate::math::constants::*;
use crate::math::transformations::coords_to_index;
use crate::render::gradient::Gradient;
use crate::simulation::grid::Grid;
use crate::ui::state::*;
use crate::undo::{UndoEvent, UndoRedo};

#[derive(SystemParam)]
pub struct EventSystemParams<'w> {
    // ---- required lifetimes: ----
    // Query<'w, 's, Entity>,
    // Res<'w, SomeResource>,
    // ResMut<'w, SomeOtherResource>,
    // Local<'s, u8>,
    // Commands<'w, 's>,
    // EventReader<'w, 's, SomeEvent>,
    // EventWriter<'w, SomeEvent>
    wall_update_ev: EventWriter<'w, UpdateWalls>,
    reset_ev: EventWriter<'w, Reset>,
    undo_ev: EventWriter<'w, UndoEvent>,
    save_ev: EventWriter<'w, Save>,
    load_ev: EventWriter<'w, Load>,
}

type AllRectWallsMut<'w, 's> = Query<'w, 's, (Entity, &'static mut RectWall)>;
type AllRectWallsSelected<'w, 's> = Query<'w, 's, (Entity, &'static mut RectWall), With<Selected>>;
type AllRectWallsMenuSelected<'w, 's> =
    Query<'w, 's, (Entity, &'static mut RectWall), With<MenuSelected>>;
type AllRectWalls<'w, 's> = Query<'w, 's, &'static RectWall>;

type AllCircWallsMut<'w, 's> = Query<'w, 's, (Entity, &'static mut CircWall)>;
type AllCircWallsSelected<'w, 's> = Query<'w, 's, (Entity, &'static mut CircWall), With<Selected>>;
type AllCircWallsMenuSelected<'w, 's> =
    Query<'w, 's, (Entity, &'static mut CircWall), With<MenuSelected>>;
type AllCircWalls<'w, 's> = Query<'w, 's, &'static CircWall>;

type AllSourcesMut<'w, 's> = Query<'w, 's, (Entity, &'static mut Source)>;
type AllSourcesSelected<'w, 's> = Query<'w, 's, (Entity, &'static mut Source), With<Selected>>;
type AllSourcesMenuSelected<'w, 's> =
    Query<'w, 's, (Entity, &'static mut Source), With<MenuSelected>>;
type AllSources<'w, 's> = Query<'w, 's, &'static Source>;

type AllMicsMut<'w, 's> = Query<'w, 's, (Entity, &'static mut Microphone)>;
type AllMicsSelected<'w, 's> = Query<'w, 's, (Entity, &'static mut Microphone), With<Selected>>;
type AllMicsMenuSelected<'w, 's> =
    Query<'w, 's, (Entity, &'static mut Microphone), With<MenuSelected>>;
type AllMics<'w, 's> = Query<'w, 's, &'static Microphone>;

#[derive(SystemParam)]
pub struct QuerySystemParams<'w, 's> {
    rect_wall_set: ParamSet<
        'w,
        's,
        (
            AllRectWallsMut<'w, 's>,
            AllRectWallsSelected<'w, 's>,
            AllRectWallsMenuSelected<'w, 's>,
            AllRectWalls<'w, 's>,
        ),
    >,
    circ_wall_set: ParamSet<
        'w,
        's,
        (
            AllCircWallsMut<'w, 's>,
            AllCircWallsSelected<'w, 's>,
            AllCircWallsMenuSelected<'w, 's>,
            AllCircWalls<'w, 's>,
        ),
    >,
    source_set: ParamSet<
        'w,
        's,
        (
            AllSourcesMut<'w, 's>,
            AllSourcesSelected<'w, 's>,
            AllSourcesMenuSelected<'w, 's>,
            AllSources<'w, 's>,
        ),
    >,
    mic_set: ParamSet<
        'w,
        's,
        (
            AllMicsMut<'w, 's>,
            AllMicsSelected<'w, 's>,
            AllMicsMenuSelected<'w, 's>,
            AllMics<'w, 's>,
        ),
    >,
}

pub fn draw_egui(
    mut commands: Commands,
    mut windows: Query<&mut Window>,
    diagnostics: Res<DiagnosticsStore>,
    mut pixel_buffers: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut grid: ResMut<Grid>,
    mut gradient: ResMut<Gradient>,
    mut events: EventSystemParams,
    sets: QuerySystemParams,
    mut dock_state: ResMut<DockState>,
    mut fft_mic: ResMut<FftMicrophone>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    sim_time: ResMut<SimTime>,
) {
    let QuerySystemParams {
        mut rect_wall_set,
        mut circ_wall_set,
        mut source_set,
        mut mic_set,
    } = sets;

    let ctx = egui_context.ctx_mut();
    egui_extras::install_image_loaders(ctx);

    let images = [
        (
            ToolType::PlaceSource,
            egui::include_image!("../../assets/place_source.png"),
        ),
        (
            ToolType::MoveSource,
            egui::include_image!("../../assets/move_source.png"),
        ),
        (
            ToolType::DrawWall,
            egui::include_image!("../../assets/draw_wall.png"),
        ),
        (
            ToolType::ResizeWall,
            egui::include_image!("../../assets/resize_wall.png"),
        ),
        (
            ToolType::MoveWall,
            egui::include_image!("../../assets/move_wall.png"),
        ),
        (
            ToolType::PlaceMic,
            egui::include_image!("../../assets/place_mic.png"),
        ),
        (
            ToolType::MoveMic,
            egui::include_image!("../../assets/move_mic.png"),
        ),
    ];

    let key = if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    };

    if ui_state.show_help {
        egui::Window::new("Keybinds")
            .open(&mut ui_state.show_help)
            .default_size(Vec2::new(400., 400.))
            .resizable(false)
            .collapsible(false)
            .constrain(true)
            .show(ctx, |ui| {
                // TODO: add links to documentation/user manual

                ui.heading("Keybinds");

                TableBuilder::new(ui)
                    .resizable(false)
                    .striped(true)
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Action");
                        });
                        header.col(|ui| {
                            ui.strong("Keybind");
                        });
                    })
                    .body(|mut body| {
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Delete Selected");
                            });
                            row.col(|ui| {
                                ui.label("Backspace or Delete");
                            });
                        });
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Undo");
                            });
                            row.col(|ui| {
                                ui.label(format!("{key}+Z"));
                            });
                        });
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Redo");
                            });
                            row.col(|ui| {
                                ui.label(format!("{key}+Shift+Z"));
                            });
                        });
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Copy Selected");
                            });
                            row.col(|ui| {
                                ui.label(format!("{key}+C"));
                            });
                        });
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Paste Clipboard");
                            });
                            row.col(|ui| {
                                ui.label(format!("{key}+V"));
                            });
                        });
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Save Current Simulation");
                            });
                            row.col(|ui| {
                                ui.label(format!("{key}+S"));
                            });
                        });
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Load Previous Simulation");
                            });
                            row.col(|ui| {
                                ui.label(format!("{key}+L"));
                            });
                        });
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Snap to Grid");
                            });
                            row.col(|ui| {
                                ui.label(format!("{key} + Move or Resize Wall"));
                            });
                        });
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Play/Pause");
                            });
                            row.col(|ui| {
                                ui.label("Space");
                            });
                        });
                    });
            });
    }

    let mut enable_spectrogram = ui_state.enable_spectrogram;
    if ui_state.show_preferences {
        egui::Window::new("Preferences")
            .open(&mut ui_state.show_preferences)
            .default_size(Vec2::new(400., 400.))
            .resizable(false)
            .collapsible(false)
            .constrain(true)
            .show(ctx, |ui| {
                ui.set_min_width(250.);
                ui.heading("Experimental Settings");

                let mut window = windows.single_mut();

                let mut is_vsync_enabled = matches!(window.present_mode, PresentMode::AutoVsync);
                ui.label("WARNING: Disabling the Vsync option may potentially trigger seizures for people with photosensitive epilepsy. Discretion is advised.");
                ui.checkbox(&mut is_vsync_enabled, "Vsync enabled");
                window.present_mode = if is_vsync_enabled {
                    PresentMode::AutoVsync
                } else {
                    PresentMode::AutoNoVsync
                };

                ui.checkbox(&mut enable_spectrogram, "Spectrogram enabled");
            });

        ui_state.enable_spectrogram = enable_spectrogram;
    }

    if ui_state.show_about {
        egui::Window::new("About")
            .open(&mut ui_state.show_about)
            .default_size(Vec2::new(400., 400.))
            .resizable(false)
            .collapsible(false)
            .constrain(true)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("wavefront");
                    ui.strong(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                });

                ui.add_space(5.);
                ui.label("A wave simulation tool using the Transmission Line Matrix method.");

                ui.add_space(10.);

                ui.heading("Created by");
                ui.hyperlink("https://github.com/JonathanKr");
                ui.hyperlink("https://github.com/ecrax");

                ui.add_space(5.);
                ui.heading("Source");
                //TODO: maybe add links to papers?
                ui.hyperlink("https://github.com/nichilum/wavefront");
            });
    }

    egui::TopBottomPanel::top("top_menu")
        .frame(
            Frame::default()
                .inner_margin(Margin {
                    left: 5.,
                    right: 5.,
                    top: 2.,
                    bottom: 2.,
                })
                .fill(Color32::from_rgb(15, 15, 15)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.visuals_mut().button_frame = false;
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("Save").shortcut_text(format!("{key}+S")))
                        .on_hover_text("Save the current state of the simulation")
                        .clicked()
                    {
                        ui.close_menu();
                        events.save_ev.send(Save);
                    }

                    if ui
                        .add(egui::Button::new("Load").shortcut_text(format!("{key}+L")))
                        .on_hover_text("Load a previously saved state of the simulation")
                        .clicked()
                    {
                        ui.close_menu();
                        events.load_ev.send(Load);
                    }

                    if ui
                        .button("Screenshot")
                        .on_hover_text("Save a screenshot of the simulation")
                        .clicked()
                    {
                        ui.close_menu();

                        let mut pixels: Vec<u8> = Vec::new();

                        for y in
                            ui_state.boundary_width..(SIMULATION_WIDTH + ui_state.boundary_width)
                        {
                            for x in ui_state.boundary_width
                                ..(SIMULATION_HEIGHT + ui_state.boundary_width)
                            {
                                let current_index = coords_to_index(x, y, ui_state.boundary_width);
                                if grid.wall_cache[current_index].is_wall {
                                    let mut reflection_factor =
                                        grid.wall_cache[current_index].reflection_factor;
                                    if reflection_factor == 0. {
                                        reflection_factor = 1.;
                                    }
                                    pixels.push((reflection_factor * 255.) as u8);
                                    pixels.push((reflection_factor * 255.) as u8);
                                    pixels.push((reflection_factor * 255.) as u8);
                                } else {
                                    let pressure = grid.pressure[current_index];

                                    let color = gradient.at(pressure, ui_state.gradient_contrast);

                                    // gamma correction to match the brightness/contrast of the simulation
                                    pixels.push(
                                        ((color.r() as f32 / 255.).powf(1. / 2.2) * 255.) as u8,
                                    );
                                    pixels.push(
                                        ((color.g() as f32 / 255.).powf(1. / 2.2) * 255.) as u8,
                                    );
                                    pixels.push(
                                        ((color.b() as f32 / 255.).powf(1. / 2.2) * 255.) as u8,
                                    );
                                }
                            }
                        }

                        let mut data = Vec::new();
                        let encoder = image::codecs::png::PngEncoder::new(&mut data);

                        let image =
                            image::RgbImage::from_raw(SIMULATION_WIDTH, SIMULATION_HEIGHT, pixels)
                                .expect("could not create image");

                        image
                            .write_with_encoder(encoder)
                            .expect("could not write image");

                        commands
                            .dialog()
                            .add_filter("PNG", &["png"])
                            .set_file_name("screenshot.png")
                            .set_directory("./")
                            .set_title("Select a file to save to")
                            .save_file::<SaveFileContents>(data);
                    }

                    if ui
                        .add(egui::Button::new("Quit").shortcut_text("Esc"))
                        .on_hover_text("Quit the application")
                        .clicked()
                    {
                        app_exit_events.send(bevy::app::AppExit);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui
                        .add(egui::Button::new("Undo").shortcut_text(format!("{key}+Z")))
                        .clicked()
                    {
                        ui.close_menu();
                        events.undo_ev.send(UndoEvent(UndoRedo::Undo));
                    }
                    if ui
                        .add(egui::Button::new("Redo").shortcut_text(format!("{key}+Shift+Z")))
                        .clicked()
                    {
                        ui.close_menu();
                        events.undo_ev.send(UndoEvent(UndoRedo::Redo));
                    }
                    if ui.button("Preferences").clicked() {
                        ui_state.show_preferences = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Keybinds").clicked() {
                        ui_state.show_help = true;
                        ui.close_menu();
                    }
                    if ui.button("About").clicked() {
                        ui_state.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

    // Side Panel (Sources, Mic, Walls, Tool Options, Settings)
    egui::SidePanel::left("left_panel")
        .default_width(420.)
        .resizable(false)
        .show(ctx, |ui| {
            ui_state.tools_enabled = !ui.rect_contains_pointer(ui.available_rect_before_wrap())
                && !ui_state.render_abc_area;

            ui.spacing_mut().slider_width = 200.0;

            ui.add_space(3.);
            egui::Grid::new("header_grid")
                .min_col_width(420. / 2.)
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
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut source.x)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_WIDTH as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    events.reset_ev.send(Reset);
                                }
                                ui.add_space(10.);
                                ui.label("y:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut source.y)
                                            .speed(1)
                                            .clamp_range(0.0..=SIMULATION_HEIGHT as f32 - 1.),
                                    )
                                    .changed()
                                {
                                    events.reset_ev.send(Reset);
                                }
                            });

                            egui::ComboBox::from_label("Waveform")
                                .selected_text(format!("{}", source.source_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut source.source_type,
                                        SourceType::default_sin(),
                                        "Sinus",
                                    );
                                    ui.selectable_value(
                                        &mut source.source_type,
                                        SourceType::default_gauss(),
                                        "Gauss",
                                    );
                                    ui.selectable_value(
                                        &mut source.source_type,
                                        SourceType::default_noise(),
                                        "White Noise",
                                    );
                                });

                            match &mut source.source_type {
                                SourceType::Sin {
                                    phase,
                                    frequency,
                                    amplitude,
                                } => {
                                    if ui
                                        .add(
                                            egui::Slider::new(frequency, 0.0..=20000.0)
                                                .text("Frequency (Hz)"),
                                        )
                                        .changed()
                                    {
                                        events.reset_ev.send(Reset);
                                    }
                                    if ui
                                        .add(
                                            egui::Slider::new(amplitude, 0.0..=25.0)
                                                .text("Amplitude"),
                                        )
                                        .changed()
                                    {
                                        events.reset_ev.send(Reset);
                                    }
                                    if ui
                                        .add(
                                            egui::Slider::new(phase, 0.0..=360.0).text("Phase (°)"),
                                        )
                                        .changed()
                                    {
                                        events.reset_ev.send(Reset);
                                    }
                                }
                                SourceType::Gauss {
                                    phase,
                                    frequency,
                                    amplitude,
                                } => {
                                    if ui
                                        .add(
                                            egui::Slider::new(frequency, 0.0..=20000.0)
                                                .text("Frequency (Hz)"),
                                        )
                                        .changed()
                                    {
                                        events.reset_ev.send(Reset);
                                    }
                                    if ui
                                        .add(
                                            egui::Slider::new(amplitude, 0.0..=25.0)
                                                .text("Amplitude"),
                                        )
                                        .changed()
                                    {
                                        events.reset_ev.send(Reset);
                                    }
                                    if ui
                                        .add(
                                            egui::Slider::new(phase, 0.0..=360.0).text("Phase (°)"),
                                        )
                                        .changed()
                                    {
                                        events.reset_ev.send(Reset);
                                    }
                                }
                                SourceType::WhiteNoise { amplitude } => {
                                    if ui
                                        .add(
                                            egui::Slider::new(amplitude, 0.0..=25.0)
                                                .text("Amplitude"),
                                        )
                                        .changed()
                                    {
                                        events.reset_ev.send(Reset);
                                    }
                                }
                            }

                            if ui
                                .add(egui::Button::new("Delete").fill(Color32::DARK_RED))
                                .clicked()
                            {
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

                    if !source_set.p0().is_empty() {
                        ui.separator();
                    }

                    // Microphones
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
                            if ui
                                .add(egui::Button::new("Delete").fill(Color32::DARK_RED))
                                .clicked()
                            {
                                if let Some(current_id) = fft_mic.mic_id {
                                    if current_id == mic.id {
                                        fft_mic.mic_id = None;
                                    }
                                }
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

                    if !mic_set.p0().is_empty() {
                        ui.separator();
                    }

                    // Rect Walls
                    let mut rect_binding = rect_wall_set.p0();
                    let mut wall_vec = rect_binding.iter_mut().collect::<Vec<_>>();
                    wall_vec.sort_by_cached_key(|(_, wall)| wall.id);

                    wall_vec.iter_mut().for_each(|(entity, ref mut wall)| {
                        let collapse =
                            ui.collapsing(format!("Rectangular Wall {}", wall.id), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("x:");
                                    if ui
                                        .add(
                                            egui::DragValue::new(&mut wall.rect.min.x)
                                                .speed(1)
                                                .clamp_range(0..=SIMULATION_WIDTH - 1),
                                        )
                                        .changed()
                                    {
                                        commands.entity(*entity).try_insert(WResize::Menu);
                                        if wall.rect.min.x > wall.rect.max.x - 1 {
                                            wall.rect.min.x = wall.rect.max.x - 1;
                                        }
                                        events.reset_ev.send(Reset);
                                    }
                                    ui.add_space(10.);
                                    ui.label("y:");
                                    if ui
                                        .add(
                                            egui::DragValue::new(&mut wall.rect.min.y)
                                                .speed(1)
                                                .clamp_range(0..=SIMULATION_HEIGHT - 1),
                                        )
                                        .changed()
                                    {
                                        commands.entity(*entity).try_insert(WResize::Menu);
                                        if wall.rect.min.y > wall.rect.max.y - 1 {
                                            wall.rect.min.y = wall.rect.max.y - 1;
                                        }
                                        events.reset_ev.send(Reset);
                                    }
                                    ui.add_space(10.);
                                    ui.label("Top Left Corner");
                                });

                                ui.horizontal(|ui| {
                                    ui.label("x:");
                                    if ui
                                        .add(
                                            egui::DragValue::new(&mut wall.rect.max.x)
                                                .speed(1)
                                                .clamp_range(0..=SIMULATION_WIDTH - 1),
                                        )
                                        .changed()
                                    {
                                        commands.entity(*entity).try_insert(WResize::Menu);
                                        if wall.rect.max.x < wall.rect.min.x + 1 {
                                            wall.rect.max.x = wall.rect.min.x + 1;
                                        }
                                        events.reset_ev.send(Reset);
                                    }
                                    ui.add_space(10.);
                                    ui.label("y:");
                                    if ui
                                        .add(
                                            egui::DragValue::new(&mut wall.rect.max.y)
                                                .speed(1)
                                                .clamp_range(0..=SIMULATION_HEIGHT - 1),
                                        )
                                        .changed()
                                    {
                                        commands.entity(*entity).try_insert(WResize::Menu);
                                        if wall.rect.max.y < wall.rect.min.y + 1 {
                                            wall.rect.max.y = wall.rect.min.y + 1;
                                        }
                                        events.reset_ev.send(Reset);
                                    }
                                    ui.add_space(10.);
                                    ui.label("Bottom Right Corner");
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

                                if ui
                                    .add(
                                        // 0.01 because rendering then draws white
                                        egui::Slider::new(&mut wall.reflection_factor, 0.01..=1.0)
                                            .text("Wall Reflection Factor"),
                                    )
                                    .changed()
                                {
                                    events.reset_ev.send(Reset);
                                }

                                if ui.checkbox(&mut wall.is_hollow, "Hollow").changed() {
                                    events.wall_update_ev.send(UpdateWalls);
                                    events.reset_ev.send(Reset);
                                };

                                if ui
                                    .add(egui::Button::new("Delete").fill(Color32::DARK_RED))
                                    .clicked()
                                {
                                    commands.entity(*entity).despawn();
                                    events.wall_update_ev.send(UpdateWalls);
                                    events.reset_ev.send(Reset);
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

                    // Circ Walls
                    let mut circ_binding = circ_wall_set.p0();
                    let mut wall_vec = circ_binding.iter_mut().collect::<Vec<_>>();
                    wall_vec.sort_by_cached_key(|(_, wall)| wall.id);

                    wall_vec.iter_mut().for_each(|(entity, ref mut wall)| {
                        let collapse = ui.collapsing(format!("Circular Wall {}", wall.id), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("x:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wall.center.x)
                                            .speed(1)
                                            .clamp_range(0..=SIMULATION_WIDTH - 1),
                                    )
                                    .changed()
                                {
                                    commands.entity(*entity).try_insert(WResize::Menu);
                                    events.reset_ev.send(Reset);
                                }
                                ui.add_space(10.);
                                ui.label("y:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wall.center.y)
                                            .speed(1)
                                            .clamp_range(0..=SIMULATION_HEIGHT - 1),
                                    )
                                    .changed()
                                {
                                    commands.entity(*entity).try_insert(WResize::Menu);
                                    events.reset_ev.send(Reset);
                                }
                                ui.add_space(10.);
                                ui.label("Center");

                                ui.add_space(5.);
                                ui.add(egui::Separator::default().vertical());
                                ui.add_space(5.);

                                ui.label("Radius:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut wall.radius)
                                            .speed(1)
                                            .clamp_range(1..=1000),
                                    )
                                    .changed()
                                {
                                    commands.entity(*entity).try_insert(WResize::Menu);
                                    events.reset_ev.send(Reset);
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "Radius: {:.3} m",
                                    wall.radius as f32 * ui_state.delta_l
                                ));
                            });

                            if wall.is_hollow {
                                if ui
                                    .add(
                                        egui::Slider::new(
                                            &mut wall.open_circ_segment,
                                            0f32..=180f32.to_radians(),
                                        )
                                        .text("Open Circle Arc"),
                                    )
                                    .changed()
                                {
                                    commands.entity(*entity).try_insert(WResize::Menu);
                                    events.reset_ev.send(Reset);
                                }

                                if ui
                                    .add(
                                        egui::Slider::new(
                                            &mut wall.rotation_angle,
                                            0f32..=360f32.to_radians(),
                                        )
                                        .text("Rotation Angle"),
                                    )
                                    .changed()
                                {
                                    commands.entity(*entity).try_insert(WResize::Menu);
                                    events.reset_ev.send(Reset);
                                }
                            }

                            if ui
                                .add(
                                    egui::Slider::new(&mut wall.reflection_factor, 0.01..=1.0)
                                        .text("Wall Reflection Factor"),
                                )
                                .changed()
                            {
                                events.reset_ev.send(Reset);
                            }

                            if ui.checkbox(&mut wall.is_hollow, "Hollow Wall").changed() {
                                events.wall_update_ev.send(UpdateWalls);
                                events.reset_ev.send(Reset);
                            };

                            if ui
                                .add(egui::Button::new("Delete").fill(Color32::DARK_RED))
                                .clicked()
                            {
                                commands.entity(*entity).despawn();
                                events.wall_update_ev.send(UpdateWalls);
                                events.reset_ev.send(Reset);
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
                ui.add_space(3.);
                ui.heading("General Settings");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui
                        .button(if ui_state.is_running { "Stop" } else { "Start" })
                        .clicked()
                    {
                        ui_state.is_running = !ui_state.is_running;
                    }

                    if ui.button("Reset").clicked() {
                        events.reset_ev.send(Reset);
                    }

                    if ui
                        .add(egui::Button::new("Delete all").fill(Color32::DARK_RED))
                        .clicked()
                    {
                        for (e, _) in source_set.p0().iter() {
                            commands.entity(e).despawn();
                        }
                        for (e, _) in rect_wall_set.p0().iter() {
                            commands.entity(e).despawn();
                        }
                        for (e, _) in circ_wall_set.p0().iter() {
                            commands.entity(e).despawn();
                        }
                        for (e, _) in mic_set.p0().iter() {
                            commands.entity(e).despawn();
                        }

                        fft_mic.mic_id = None;

                        grid.reset_cells(ui_state.boundary_width);
                        events.wall_update_ev.send(UpdateWalls);
                    }

                    ui.checkbox(&mut ui_state.reset_on_change, "Reset on change")
                });

                if ui
                    .add(
                        egui::Slider::new(&mut ui_state.delta_l, 0.0..=10.0)
                            .text("Delta L (m)")
                            .logarithmic(true),
                    )
                    .on_hover_text("Change the size of one cell in the simulation in meters.")
                    .changed()
                {
                    events.reset_ev.send(Reset);
                }

                if ui
                    .checkbox(&mut ui_state.show_plots, "Show Plots")
                    .clicked()
                {
                    for (_, mut mic) in mic_set.p0().iter_mut() {
                        mic.clear();
                    }
                }

                ui.collapsing("Gradient", |ui| {
                    ui.label("Adjust the colors used to render the simulation.");
                    ui.add_space(5.);

                    ui.horizontal(|ui| {
                        ui.color_edit_button_srgba(&mut gradient.0).on_hover_text("The color used to show negative pressure values.");
                        ui.add_space(10.);
                        ui.label("Negative");
                    });
                    ui.horizontal(|ui| {
                        ui.color_edit_button_srgba(&mut gradient.1).on_hover_text("The color used to show neutral pressure values.");
                        ui.add_space(10.);
                        ui.label("Zero");
                    });
                    ui.horizontal(|ui| {
                        ui.color_edit_button_srgba(&mut gradient.2).on_hover_text("The color used to show positive pressure values.");
                        ui.add_space(10.);
                        ui.label("Positive");
                    });

                    ui.add(
                        egui::Slider::new(&mut ui_state.gradient_contrast, 0.0..=10.0)
                            .text("Gradient Contrast"),
                    ).on_hover_text("Adjust the contrast of the gradient. (this might lead to clipping the colors)");
                });

                ui.collapsing("Boundary", |ui| {
                    ui.label("Change the outer boundary for the free field simulation.");
                    ui.add_space(5.);

                    if ui
                        .checkbox(&mut ui_state.render_abc_area, "Show absorbing boundary")
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

                    if ui
                        .add(
                            egui::Slider::new(&mut ui_state.boundary_width, 2..=200)
                                .text("Boundary Width"),
                        )
                        .on_hover_text("Change the width of the boundary. (higher values lead to slower simulation)")
                        .changed()
                    {
                        grid.reset_cells(ui_state.boundary_width);
                        grid.reset_walls(ui_state.boundary_width);
                        grid.cache_boundaries(ui_state.boundary_width);
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
                });

                ui.add_space(5.);
                ui.label(format!("Simulation Time: {:.5} ms", sim_time.time_since_start * 1000.));

                ui.add_space(5.);
            });

            // Tool Options
            egui::TopBottomPanel::bottom("tool_options_panel").show_inside(ui, |ui| {
                ui.add_space(3.);
                ui.heading("Tool Options");
                ui.separator();

                ui.set_enabled(!ui_state.render_abc_area);
                if ui_state.current_tool == ToolType::DrawWall {
                    egui::ComboBox::from_label("Select Wall Type")
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
                    ui.checkbox(&mut ui_state.wall_is_hollow, "Hollow");
                } else {
                    ui.add_space(10.);
                    ui.vertical_centered(|ui| ui.label("Select another tool to see its options"));
                }
                ui.add_space(10.);
            });
        });

    // Plot tabs
    if ui_state.show_plots {
        egui::TopBottomPanel::bottom("plot_tabs")
            .resizable(true)
            .default_height(400.)
            .max_height(ctx.screen_rect().height() / 2.)
            .frame(Frame::default().inner_margin(Margin {
                left: -1.,
                right: 0.,
                top: 0.,
                bottom: 0.,
            }))
            .show(ctx, |ui| {
                let mut binding = mic_set.p3();
                let mics = binding.iter_mut().collect::<Vec<_>>();
                let mut pb = pixel_buffers.iter_mut().nth(1).expect("two pixel buffers");

                let mut style = egui_dock::Style::from_egui(ui.style());
                style.tab_bar.bg_fill = Color32::from_rgb(27, 27, 27);

                egui_dock::DockArea::new(&mut dock_state.tree)
                    .allowed_splits(egui_dock::AllowedSplits::None)
                    .draggable_tabs(false)
                    .style(style)
                    .show_inside(
                        ui,
                        &mut PlotTabs {
                            mics: &mics[..],
                            pixel_buffer: &mut pb,
                            fft_microphone: &mut fft_mic,
                            commands: &mut commands.reborrow(),
                            enabled_spectrogram: ui_state.enable_spectrogram,
                            scaling: &mut ui_state.fft_scaling,
                            delta_t: grid.delta_t,
                        },
                    );
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

            for (tool_type, icon) in images {
                if ui
                    .add(
                        egui::Button::image(
                            egui::Image::new(icon).fit_to_exact_size(Vec2::new(25., 25.)),
                        )
                        .fill(if tool_type == ui_state.current_tool {
                            Color32::DARK_GRAY
                        } else {
                            Color32::TRANSPARENT
                        })
                        .min_size(Vec2::new(0., 35.)),
                    )
                    .on_hover_text(format!("{}", tool_type))
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
            ui_state.tool_use_enabled = ui.rect_contains_pointer(ui.min_rect().expand(20.));

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
                let text_color = Color32::WHITE;
                //menu gizmos
                if !ui_state.tools_enabled {
                    for (_, wall) in rect_wall_set.p2().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ToolType::MoveWall,
                            true,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    for (_, wall) in circ_wall_set.p2().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ToolType::MoveWall,
                            true,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    // all mics
                    for (_, mic) in mic_set.p2().iter() {
                        mic.draw_gizmo(
                            painter,
                            &ToolType::MoveMic,
                            true,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    // all sources
                    for (_, source) in source_set.p2().iter() {
                        source.draw_gizmo(
                            painter,
                            &ToolType::MoveSource,
                            true,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                } else {
                    // Tool specific gizmos
                    // all walls
                    for wall in rect_wall_set.p3().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    // selected walls
                    for (_, wall) in rect_wall_set.p1().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    // all circ walls
                    for wall in circ_wall_set.p3().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    // selected circ walls
                    for (_, wall) in circ_wall_set.p1().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    // all mics
                    for mic in mic_set.p3().iter() {
                        mic.draw_gizmo(
                            painter,
                            // &ui_state.current_tool,
                            &ToolType::PlaceMic,
                            false,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    // selected mics
                    for (_, mic) in mic_set.p1().iter() {
                        mic.draw_gizmo(
                            painter,
                            // &ui_state.current_tool,
                            &ToolType::PlaceMic,
                            true,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    // all sources
                    for source in source_set.p3().iter() {
                        source.draw_gizmo(
                            painter,
                            // &ui_state.current_tool,
                            &ToolType::PlaceSource,
                            false,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                    // selected sources
                    for (_, source) in source_set.p1().iter() {
                        source.draw_gizmo(
                            painter,
                            // &ui_state.current_tool,
                            &ToolType::PlaceSource,
                            true,
                            &ui_state.image_rect,
                            ui_state.delta_l,
                            text_color,
                        );
                    }
                }
            }
        });
}
