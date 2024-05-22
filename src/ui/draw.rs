use bevy::diagnostic::DiagnosticsStore;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_file_dialog::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::{Color32, Frame, Margin, Vec2};
use bevy_pixel_buffer::bevy_egui::EguiContexts;
use bevy_pixel_buffer::prelude::*;

use super::help::draw_help;
use super::loading::SaveFileContents;
use super::panels::general_settings::draw_general_settings;
use super::panels::outline::draw_outline;
use super::panels::tool_options::draw_tool_options;
use super::panels::tool_panel::draw_tool_panel;
use super::preferences::draw_preferences;
use super::tabs::{DockState, PlotTabs};
use crate::components::gizmo::GizmoComponent;
use crate::components::microphone::*;
use crate::components::source::*;
use crate::components::states::{MenuSelected, Selected};
use crate::components::wall::{CircWall, RectWall};
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
    pub wall_update_ev: EventWriter<'w, UpdateWalls>,
    pub reset_ev: EventWriter<'w, Reset>,
    pub undo_ev: EventWriter<'w, UndoEvent>,
    pub save_ev: EventWriter<'w, Save>,
    pub load_ev: EventWriter<'w, Load>,
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
    pub rect_wall_set: ParamSet<
        'w,
        's,
        (
            AllRectWallsMut<'w, 's>,
            AllRectWallsSelected<'w, 's>,
            AllRectWallsMenuSelected<'w, 's>,
            AllRectWalls<'w, 's>,
        ),
    >,
    pub circ_wall_set: ParamSet<
        'w,
        's,
        (
            AllCircWallsMut<'w, 's>,
            AllCircWallsSelected<'w, 's>,
            AllCircWallsMenuSelected<'w, 's>,
            AllCircWalls<'w, 's>,
        ),
    >,
    pub source_set: ParamSet<
        'w,
        's,
        (
            AllSourcesMut<'w, 's>,
            AllSourcesSelected<'w, 's>,
            AllSourcesMenuSelected<'w, 's>,
            AllSources<'w, 's>,
        ),
    >,
    pub mic_set: ParamSet<
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

pub const CTRL_KEY_TEXT: &'static str = if cfg!(target_os = "macos") {
    "Cmd"
} else {
    "Ctrl"
};

pub fn draw_egui(
    mut commands: Commands,
    mut pixel_buffers: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut grid: ResMut<Grid>,
    mut gradient: ResMut<Gradient>,
    mut events: EventSystemParams,
    mut sets: QuerySystemParams,
    mut dock_state: ResMut<DockState>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    sim_time: Res<SimTime>,
    time: Res<Time>,
    mut fixed_timestep: ResMut<Time<Fixed>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let ctx = egui_context.ctx_mut();
    egui_extras::install_image_loaders(ctx);

    if ui_state.show_help {
        draw_help(&mut ui_state, ctx);
    }

    if ui_state.show_preferences {
        let mut show_preferences = ui_state.show_preferences;

        draw_preferences(
            &mut show_preferences,
            ctx,
            &mut ui_state,
            &mut events,
            &mut grid,
            &mut pixel_buffers,
            &mut gradient,
        );

        ui_state.show_preferences = show_preferences;
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

    if ui_state.show_epilepsy_warning {
        let mut read_epilepsy_warning = ui_state.read_epilepsy_warning;
        egui::Window::new("Epilepsy Warning")
            .default_size(Vec2::new(400., 400.))
            .resizable(false)
            .collapsible(false)
            .constrain(true)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("WARNING");
                });
                ui.strong("Increasing the speed may potentially trigger seizures for people with photosensitive epilepsy. Discretion is advised.");

                if ui.checkbox(&mut read_epilepsy_warning, "Understood").clicked() {
                    ui_state.show_epilepsy_warning = false;
                }
            });
        ui_state.read_epilepsy_warning = read_epilepsy_warning;
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
                        .add(egui::Button::new("Save").shortcut_text(format!("{CTRL_KEY_TEXT}+S")))
                        .on_hover_text("Save the current state of the simulation")
                        .clicked()
                    {
                        ui.close_menu();
                        events.save_ev.send(Save);
                    }

                    if ui
                        .add(egui::Button::new("Load").shortcut_text(format!("{CTRL_KEY_TEXT}+L")))
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

                                    let color = gradient.at(pressure, -2., 2.);

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
                        .add(egui::Button::new("Undo").shortcut_text(format!("{CTRL_KEY_TEXT}+Z")))
                        .clicked()
                    {
                        ui.close_menu();
                        events.undo_ev.send(UndoEvent(UndoRedo::Undo));
                    }
                    if ui
                        .add(
                            egui::Button::new("Redo")
                                .shortcut_text(format!("{CTRL_KEY_TEXT}+Shift+Z")),
                        )
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

            draw_outline(&mut sets, &mut ui_state, ui, &mut events, &mut commands);

            draw_general_settings(
                &mut sets,
                &mut ui_state,
                ui,
                &mut events,
                &mut commands,
                &mut grid,
                diagnostics,
                &sim_time,
                &mut fixed_timestep,
            );

            draw_tool_options(&mut ui_state, ui);
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
                let mut binding = sets.mic_set.p0();

                let mut mics = binding
                    .iter_mut()
                    .map(|(_, mic)| mic.into_inner())
                    .collect::<Vec<_>>();
                mics.sort_by_cached_key(|mic| mic.id);

                let mut pb = pixel_buffers.iter_mut().nth(1).expect("two pixel buffers");

                let mut style = egui_dock::Style::from_egui(ui.style());
                style.tab_bar.bg_fill = Color32::from_rgb(27, 27, 27);

                egui_dock::DockArea::new(&mut dock_state.tree)
                    .allowed_splits(egui_dock::AllowedSplits::None)
                    .draggable_tabs(false)
                    .style(style)
                    .show_inside(
                        ui,
                        &mut PlotTabs::new(
                            &mut mics,
                            &mut pb,
                            // &mut fft_mic,
                            &mut commands.reborrow(),
                            grid.delta_t,
                            sim_time.time_since_start as f64,
                            time.delta_seconds_f64(),
                            &mut ui_state,
                        ),
                    );
            });
    }

    draw_tool_panel(&mut ui_state, &ctx);

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
                //menu gizmos
                if !ui_state.tools_enabled {
                    for (_, wall) in sets.rect_wall_set.p2().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ToolType::Move,
                            true,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                        );
                    }
                    for (_, wall) in sets.circ_wall_set.p2().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ToolType::Move,
                            true,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                        );
                    }
                    // all mics
                    for (_, mic) in sets.mic_set.p2().iter() {
                        mic.draw_gizmo(
                            painter,
                            &ToolType::Move,
                            true,
                            &ui_state.image_rect,
                            Some(&format!("{}", mic.id)),
                            ui_state.delta_l,
                        );
                    }
                    // all sources
                    for (_, source) in sets.source_set.p2().iter() {
                        source.draw_gizmo(
                            painter,
                            &ToolType::Move,
                            true,
                            &ui_state.image_rect,
                            Some(&format!("{}", source.id)),
                            ui_state.delta_l,
                        );
                    }
                } else {
                    // TODO: drawing selected gizmos on top means that the text is also drawn twice
                    // Tool specific gizmos
                    // all rect walls
                    for wall in sets.rect_wall_set.p3().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                        );
                    }
                    // selected rect walls
                    for (_, wall) in sets.rect_wall_set.p1().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                        );
                    }
                    // all circ walls
                    for wall in sets.circ_wall_set.p3().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                        );
                    }
                    // selected circ walls
                    for (_, wall) in sets.circ_wall_set.p1().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                        );
                    }
                    // all mics
                    for mic in sets.mic_set.p3().iter() {
                        mic.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            Some(&format!("{}", mic.id)),
                            ui_state.delta_l,
                        );
                    }
                    // selected mics
                    for (_, mic) in sets.mic_set.p1().iter() {
                        mic.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            Some(&format!("{}", mic.id)),
                            ui_state.delta_l,
                        );
                    }
                    // all sources
                    for source in sets.source_set.p3().iter() {
                        source.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            Some(&format!("{}", source.id)),
                            ui_state.delta_l,
                        );
                    }
                    // selected sources
                    for (_, source) in sets.source_set.p1().iter() {
                        source.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            Some(&format!("{}", source.id)),
                            ui_state.delta_l,
                        );
                    }
                }
            }
        });

    ui_state.collapse_header = false;
}
