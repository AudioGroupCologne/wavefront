use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::Color32;

use crate::components::source::*;
use crate::components::states::MenuSelected;
use crate::components::wall::WResize;
use crate::events::{Reset, UpdateWalls};
use crate::math::constants::*;
use crate::ui::draw::{EventSystemParams, QuerySystemParams};
use crate::ui::state::*;

pub fn draw_outline(
    sets: &mut QuerySystemParams,
    ui_state: &mut UiState,
    ui: &mut egui::Ui,
    events: &mut EventSystemParams,
    commands: &mut Commands,
) {
    let QuerySystemParams {
        rect_wall_set,
        circ_wall_set,
        source_set,
        mic_set,
    } = sets;

    egui::ScrollArea::vertical()
        .id_source("side_scroll_area")
        .max_height(400.)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            let binding = source_set.p1();
            let selected_source = binding.iter().next();
            let selected_source = if selected_source.is_some() {
                selected_source.unwrap().1.id as i32
            } else {
                -1_i32
            };

            let mut binding = source_set.p0();
            let mut source_vec = binding.iter_mut().collect::<Vec<_>>();
            source_vec.sort_by_cached_key(|(_, source)| source.id);

            source_vec.iter_mut().for_each(|(entity, ref mut source)| {
                let collapse = egui::CollapsingHeader::new(format!("Source {}", source.id))
                    .open(if selected_source == source.id as i32 {
                        Some(true)
                    } else if ui_state.collapse_header {
                        Some(false)
                    } else {
                        None
                    })
                    .show(ui, |ui| {
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
                                events.reset_ev.send(Reset::default());
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
                                events.reset_ev.send(Reset::default());
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
                                    events.reset_ev.send(Reset::default());
                                }
                                if ui
                                    .add(egui::Slider::new(amplitude, 0.0..=25.0).text("Amplitude"))
                                    .changed()
                                {
                                    events.reset_ev.send(Reset::default());
                                }
                                if ui
                                    .add(egui::Slider::new(phase, 0.0..=360.0).text("Phase (°)"))
                                    .changed()
                                {
                                    events.reset_ev.send(Reset::default());
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
                                    events.reset_ev.send(Reset::default());
                                }
                                if ui
                                    .add(egui::Slider::new(amplitude, 0.0..=25.0).text("Amplitude"))
                                    .changed()
                                {
                                    events.reset_ev.send(Reset::default());
                                }
                                if ui
                                    .add(egui::Slider::new(phase, 0.0..=360.0).text("Phase (°)"))
                                    .changed()
                                {
                                    events.reset_ev.send(Reset::default());
                                }
                            }
                            SourceType::WhiteNoise { amplitude } => {
                                if ui
                                    .add(egui::Slider::new(amplitude, 0.0..=25.0).text("Amplitude"))
                                    .changed()
                                {
                                    events.reset_ev.send(Reset::default());
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
                if collapse.header_response.contains_pointer() || collapse.body_response.is_some() {
                    commands.entity(*entity).try_insert(MenuSelected);
                } else {
                    commands.entity(*entity).remove::<MenuSelected>();
                }
            });

            if !source_set.p0().is_empty() {
                ui.separator();
            }

            // Microphones

            let binding = mic_set.p1();
            let selected_mic = binding.iter().next();
            let selected_mic = if selected_mic.is_some() {
                selected_mic.unwrap().1.id as i32
            } else {
                -1_i32
            };

            let mut binding = mic_set.p0();
            let mut mic_vec = binding.iter_mut().collect::<Vec<_>>();
            mic_vec.sort_by_cached_key(|(_, mic)| mic.id);

            mic_vec.iter_mut().for_each(|(entity, ref mut mic)| {
                let collapse = egui::CollapsingHeader::new(format!("Microphone {}", mic.id))
                    .open(if selected_mic == mic.id as i32 {
                        Some(true)
                    } else if ui_state.collapse_header {
                        Some(false)
                    } else {
                        None
                    })
                    .show(ui, |ui| {
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
                            commands.entity(*entity).despawn();
                        }
                    });
                if collapse.header_response.contains_pointer() || collapse.body_response.is_some() {
                    commands.entity(*entity).try_insert(MenuSelected);
                } else {
                    commands.entity(*entity).remove::<MenuSelected>();
                }
            });

            if !mic_set.p0().is_empty() {
                ui.separator();
            }

            // Rect Walls

            let binding = rect_wall_set.p1();
            let selected_rect_wall = binding.iter().next();
            let selected_rect_wall = if selected_rect_wall.is_some() {
                selected_rect_wall.unwrap().1.id as i32
            } else {
                -1_i32
            };

            let mut rect_binding = rect_wall_set.p0();
            let mut wall_vec = rect_binding.iter_mut().collect::<Vec<_>>();
            wall_vec.sort_by_cached_key(|(_, wall)| wall.id);

            wall_vec.iter_mut().for_each(|(entity, ref mut wall)| {
                let collapse = egui::CollapsingHeader::new(format!("Rectangular Wall {}", wall.id))
                    .open(if selected_rect_wall == wall.id as i32 {
                        Some(true)
                    } else if ui_state.collapse_header {
                        Some(false)
                    } else {
                        None
                    })
                    .show(ui, |ui| {
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
                                events.reset_ev.send(Reset::default());
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
                                events.reset_ev.send(Reset::default());
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
                                events.reset_ev.send(Reset::default());
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
                                events.reset_ev.send(Reset::default());
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
                            events.reset_ev.send(Reset::default());
                        }

                        if ui.checkbox(&mut wall.is_hollow, "Hollow").changed() {
                            events.wall_update_ev.send(UpdateWalls);
                            events.reset_ev.send(Reset::default());
                        };

                        if ui
                            .add(egui::Button::new("Delete").fill(Color32::DARK_RED))
                            .clicked()
                        {
                            commands.entity(*entity).despawn();
                            events.wall_update_ev.send(UpdateWalls);
                            events.reset_ev.send(Reset::default());
                        }
                    });

                if collapse.header_response.contains_pointer() || collapse.body_response.is_some() {
                    commands.entity(*entity).try_insert(MenuSelected);
                } else {
                    commands.entity(*entity).remove::<MenuSelected>();
                }
            });

            // Circ Walls

            let binding = circ_wall_set.p1();
            let selected_circ_wall = binding.iter().next();
            let selected_circ_wall = if selected_circ_wall.is_some() {
                selected_circ_wall.unwrap().1.id as i32
            } else {
                -1_i32
            };

            let mut circ_binding = circ_wall_set.p0();
            let mut wall_vec = circ_binding.iter_mut().collect::<Vec<_>>();
            wall_vec.sort_by_cached_key(|(_, wall)| wall.id);

            wall_vec.iter_mut().for_each(|(entity, ref mut wall)| {
                let collapse = egui::CollapsingHeader::new(format!("Circular Wall {}", wall.id))
                    .open(if selected_circ_wall == wall.id as i32 {
                        Some(true)
                    } else if ui_state.collapse_header {
                        Some(false)
                    } else {
                        None
                    })
                    .show(ui, |ui| {
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
                                events.reset_ev.send(Reset::default());
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
                                events.reset_ev.send(Reset::default());
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
                                events.reset_ev.send(Reset::default());
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
                                    egui::Slider::new(&mut wall.open_circ_segment, 0f32..=180f32)
                                        .text("Open Circle Arc"),
                                )
                                .changed()
                            {
                                commands.entity(*entity).try_insert(WResize::Menu);
                                events.reset_ev.send(Reset::default());
                            }

                            if ui
                                .add(
                                    egui::Slider::new(&mut wall.rotation_angle, 0f32..=360f32)
                                        .text("Rotation Angle"),
                                )
                                .changed()
                            {
                                commands.entity(*entity).try_insert(WResize::Menu);
                                events.reset_ev.send(Reset::default());
                            }
                        }

                        if ui
                            .add(
                                egui::Slider::new(&mut wall.reflection_factor, 0.01..=1.0)
                                    .text("Wall Reflection Factor"),
                            )
                            .changed()
                        {
                            events.reset_ev.send(Reset::default());
                        }

                        if ui.checkbox(&mut wall.is_hollow, "Hollow Wall").changed() {
                            events.wall_update_ev.send(UpdateWalls);
                            events.reset_ev.send(Reset::default());
                        };

                        if ui
                            .add(egui::Button::new("Delete").fill(Color32::DARK_RED))
                            .clicked()
                        {
                            commands.entity(*entity).despawn();
                            events.wall_update_ev.send(UpdateWalls);
                            events.reset_ev.send(Reset::default());
                        }
                    });

                if collapse.header_response.contains_pointer() || collapse.body_response.is_some() {
                    commands.entity(*entity).try_insert(MenuSelected);
                } else {
                    commands.entity(*entity).remove::<MenuSelected>();
                }
            });
        });
}
