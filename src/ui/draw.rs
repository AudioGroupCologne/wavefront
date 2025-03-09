use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::{Color32, Frame, Margin, Vec2};
use bevy_pixel_buffer::bevy_egui::EguiContexts;
use bevy_pixel_buffer::prelude::*;
use egui::ImageSource;
use egui_plot::{Line, Plot, PlotBounds, PlotPoints};

use crate::components::gizmo::GizmoComponent;
use crate::components::microphone::*;
use crate::components::source::*;
use crate::components::states::{MenuSelected, Selected};
use crate::components::wall::{CircWall, RectWall};
use crate::events::{LoadScene, LoadWav, New, Reset, Save, UpdateWalls};
use crate::math::constants::*;
use crate::render::gradient::Gradient;
use crate::simulation::grid::Grid;
use crate::ui::state::*;
use crate::undo::UndoEvent;

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
    pub load_scene_ev: EventWriter<'w, LoadScene>,
    pub load_wav_ev: EventWriter<'w, LoadWav>,
    pub new_ev: EventWriter<'w, New>,
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

pub const CTRL_KEY_TEXT: &str = if cfg!(target_os = "macos") {
    "Cmd"
} else {
    "Ctrl"
};

pub const IMAGES: [ImageSource; 4] = [
    egui::include_image!("../../assets/select.png"),
    egui::include_image!("../../assets/place.png"),
    egui::include_image!("../../assets/move.png"),
    egui::include_image!("../../assets/resize_wall.png"),
];

pub fn draw_egui(
    mut commands: Commands,
    pixel_buffer: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut grid: ResMut<Grid>,
    gradient: ResMut<Gradient>,
    mut events: EventSystemParams,
    sets: QuerySystemParams,
    sim_time: Res<SimTime>,
    diagnostics: Res<DiagnosticsStore>,
    mut tool_settings_height: Local<f32>,
) {
    let QuerySystemParams {
        mut rect_wall_set,
        mut circ_wall_set,
        mut source_set,
        mut mic_set,
    } = sets;

    let ctx = egui_context.ctx_mut();
    egui_extras::install_image_loaders(ctx);
    // disable window shadows
    ctx.style_mut(|style| style.visuals.window_shadow = egui::epaint::Shadow::NONE);
    // ctx.set_zoom_factor(2.0);
    // println!("zoom factor: {}", ctx.zoom_factor());

    // Tool Options
    // egui::TopBottomPanel::bottom("tool_options_panel").show(ctx, |ui| {
    //     *tool_settings_height = ui.available_height();

    //     ui.add_space(3.);
    //     ui.heading("Tool Options");
    //     ui.separator();

    //     if ui_state.render_abc_area {
    //         ui.disable();
    //     }

    //     match ui_state.current_tool {
    //         ToolType::Place(_) => {
    //             egui::ComboBox::from_label("Select object to place")
    //                 .selected_text(format!("{}", ui_state.cur_place_type))
    //                 .show_ui(ui, |ui| {
    //                     ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    //                     ui.selectable_value(
    //                         &mut ui_state.cur_place_type,
    //                         PlaceType::Source,
    //                         "Source",
    //                     );
    //                     ui.selectable_value(
    //                         &mut ui_state.cur_place_type,
    //                         PlaceType::Mic,
    //                         "Microphone",
    //                     );
    //                     ui.selectable_value(
    //                         &mut ui_state.cur_place_type,
    //                         PlaceType::RectWall,
    //                         "Rectangular Wall",
    //                     );
    //                     ui.selectable_value(
    //                         &mut ui_state.cur_place_type,
    //                         PlaceType::CircWall,
    //                         "Circular Wall",
    //                     );
    //                 });

    //             if matches!(
    //                 ui_state.cur_place_type,
    //                 PlaceType::RectWall | PlaceType::CircWall
    //             ) {
    //                 ui.add(
    //                     egui::Slider::new(&mut ui_state.wall_reflection_factor, 0.0..=1.0)
    //                         .text("Reflection factor"),
    //                 );
    //                 ui.checkbox(&mut ui_state.wall_is_hollow, "Hollow");
    //             }
    //             ui_state.current_tool = ToolType::Place(ui_state.cur_place_type);
    //         }
    //         _ => {
    //             ui.add_space(10.);
    //             ui.vertical_centered(|ui| ui.label("Select another tool to see its options"));
    //         }
    //     }

    //     ui.add_space(10.);
    // });

    // Tool Panel
    egui::TopBottomPanel::bottom("tool_panel")
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
        // .default_width(35.)
        .resizable(false)
        .show(ctx, |ui| {
            if !ui_state.tools_enabled {
                ui.disable();
            }

            let select_icon = &IMAGES[0];
            let place_icon = &IMAGES[1];
            let move_icon = &IMAGES[2];
            //let resize_wall_icon = &IMAGES[3];

            ui.horizontal(|ui| {
                ui.columns(3, |columns| {
                    columns[0].vertical_centered(|ui| {
                        if ui
                            .add(
                                egui::Button::image(
                                    egui::Image::new(select_icon.clone())
                                        .fit_to_exact_size(Vec2::new(50., 50.)),
                                )
                                .fill(if matches!(ui_state.current_tool, ToolType::Select) {
                                    Color32::DARK_GRAY
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .min_size(Vec2::new(150., 150.))
                                .rounding(15.),
                            )
                            .on_hover_text(format!("{}", ToolType::Select))
                            .clicked()
                        {
                            ui_state.current_tool = ToolType::Select;
                        }
                    });
                    columns[1].vertical_centered(|ui| {
                        if ui
                            .add(
                                egui::Button::image(
                                    // TODO: change image depending on cur_place_type??
                                    egui::Image::new(place_icon.clone())
                                        .fit_to_exact_size(Vec2::new(50., 50.)),
                                )
                                .fill(if matches!(ui_state.current_tool, ToolType::Place(..)) {
                                    Color32::DARK_GRAY
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .min_size(Vec2::new(150., 150.))
                                .rounding(15.),
                            )
                            .on_hover_text(format!("{}", ToolType::Place(PlaceType::Source)))
                            .clicked()
                        {
                            ui_state.current_tool = ToolType::Place(ui_state.cur_place_type);
                        }
                    });
                    columns[2].vertical_centered(|ui| {
                        if ui
                            .add(
                                egui::Button::image(
                                    egui::Image::new(move_icon.clone())
                                        .fit_to_exact_size(Vec2::new(50., 50.)),
                                )
                                .fill(if matches!(ui_state.current_tool, ToolType::Edit) {
                                    Color32::DARK_GRAY
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .min_size(Vec2::new(150., 150.))
                                .rounding(15.),
                            )
                            .on_hover_text(format!("{}", ToolType::Edit))
                            .clicked()
                        {
                            ui_state.current_tool = ToolType::Edit;
                        }
                    });
                });
            });
        });

    // Quick Settings
    egui::TopBottomPanel::bottom("quick_settings_bottom_panel").show(ctx, |ui| {
        ui.add_space(12.);
        ui.horizontal(|ui| {
            ui.columns(2, |columns| {
                columns[0].vertical_centered(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(if ui_state.is_running {
                                    "Stop"
                                } else {
                                    "Start"
                                })
                                .size(60.),
                            )
                            .min_size(Vec2::new(200., 100.))
                            .rounding(10.),
                        )
                        .clicked()
                    {
                        ui_state.is_running = !ui_state.is_running;
                    }
                });
                columns[1].vertical_centered(|ui| {
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Reset").size(60.))
                                .min_size(Vec2::new(200., 100.))
                                .rounding(10.),
                        )
                        .clicked()
                    {
                        events.reset_ev.send(Reset { force: true });
                    }
                });
            });
        });

        ui.add_space(5.);

        if cfg!(debug_assertions) {
            ui.horizontal(|ui| {
                ui.label(format!("Time: {:.5} ms", sim_time.time_since_start * 1000.));

                ui.add(egui::Separator::default().vertical());
                ui.label(format!(
                    "Size: {:.5} m",
                    ui_state.delta_l * SIMULATION_WIDTH as f32
                ));

                ui.add(egui::Separator::default().vertical());
                ui.label(format!(
                    "FPS: {:.1}",
                    diagnostics
                        .get(&FrameTimeDiagnosticsPlugin::FPS)
                        .and_then(|fps| fps.smoothed())
                        .unwrap_or(0.0)
                ));
            });

            ui.add_space(5.);
        }
    });

    egui::TopBottomPanel::bottom("plot_panel")
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
        .default_height(420.)
        .resizable(false)
        .show(ctx, |ui| {
            let scroll_volume_plot = ui_state.scroll_volume_plot;

            let mut binding = mic_set.p0();

            let mut mics = binding
                .iter_mut()
                .map(|(_, mic)| mic.into_inner())
                .collect::<Vec<_>>();
            mics.sort_by_cached_key(|mic| mic.id);

            Plot::new("mic_plot")
                .allow_zoom([!scroll_volume_plot, !scroll_volume_plot])
                .allow_drag(!scroll_volume_plot)
                .allow_scroll(!scroll_volume_plot)
                .x_axis_label("Simulation time (ms)")
                .y_axis_label("Amplitude")
                .label_formatter(|_, value| {
                    format!("Amplitude: {:.2}\nTime: {:.4} ms", value.y, value.x)
                })
                .legend(egui_plot::Legend::default())
                .show(ui, |plot_ui| {
                    if scroll_volume_plot {
                        let highest_x = sim_time.time_since_start as f64;
                        let highest_y = mics
                            .iter()
                            .map(|mic| mic.record.last().unwrap_or(&[0., 0.])[1])
                            .reduce(f64::max)
                            .unwrap_or(0.)
                            .abs();

                        if highest_y > ui_state.highest_y_volume_plot {
                            ui_state.highest_y_volume_plot = highest_y;
                        }

                        plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                            [
                                highest_x * 1000. - 5.,
                                -(ui_state.highest_y_volume_plot + 0.2),
                            ],
                            [highest_x * 1000., ui_state.highest_y_volume_plot + 0.2],
                        ));
                    }

                    for mic in &mut *mics {
                        let values = mic.record.iter().map(|x| [x[0] * 1000., x[1]]).collect();
                        let points = PlotPoints::new(values);
                        let line = Line::new(points);
                        plot_ui.line(line.name(format!(
                            "Microphone {} (x: {}, y: {})",
                            mic.id, mic.x, mic.y
                        )));
                    }
                });
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

            let texture = pixel_buffer.egui_texture();
            // let image = ui.image(egui::load::SizedTexture::new(texture.id, texture.size));

            let image = ui.add(
                egui::Image::new(egui::load::SizedTexture::new(texture.id, texture.size))
                    .shrink_to_fit(),
            );

            ui_state.image_rect = image.rect;

            // Gizmos

            if !ui_state.render_abc_area && !ui_state.hide_gizmos {
                let painter = ui.painter();
                //menu gizmos
                if !ui_state.tools_enabled {
                    for (_, wall) in rect_wall_set.p2().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ToolType::Edit,
                            true,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    for (_, wall) in circ_wall_set.p2().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ToolType::Edit,
                            true,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    // all mics
                    for (_, mic) in mic_set.p2().iter() {
                        mic.draw_gizmo(
                            painter,
                            &ToolType::Edit,
                            true,
                            &ui_state.image_rect,
                            Some(&format!("{}", mic.id)),
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    // all sources
                    for (_, source) in source_set.p2().iter() {
                        source.draw_gizmo(
                            painter,
                            &ToolType::Edit,
                            true,
                            &ui_state.image_rect,
                            Some(&format!("{}", source.id)),
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                } else {
                    // TODO: drawing selected gizmos on top means that the text is also drawn twice
                    // Tool specific gizmos
                    // all rect walls
                    for wall in rect_wall_set.p3().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    // selected rect walls
                    for (_, wall) in rect_wall_set.p1().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    // all circ walls
                    for wall in circ_wall_set.p3().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    // selected circ walls
                    for (_, wall) in circ_wall_set.p1().iter() {
                        wall.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            None,
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    // all mics
                    for mic in mic_set.p3().iter() {
                        mic.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            Some(&format!("{}", mic.id)),
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    // selected mics
                    for (_, mic) in mic_set.p1().iter() {
                        mic.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            Some(&format!("{}", mic.id)),
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    // all sources
                    for source in source_set.p3().iter() {
                        source.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            false,
                            &ui_state.image_rect,
                            Some(&format!("{}", source.id)),
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                    // selected sources
                    for (_, source) in source_set.p1().iter() {
                        source.draw_gizmo(
                            painter,
                            &ui_state.current_tool,
                            true,
                            &ui_state.image_rect,
                            Some(&format!("{}", source.id)),
                            ui_state.delta_l,
                            *gradient,
                        );
                    }
                }
            }
        });
}
