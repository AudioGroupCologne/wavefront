use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::microphone::Microphone;
use crate::components::source::{Source, SourceType};
use crate::components::states::{Drag, Overlay, Selected};
use crate::components::wall::{Wall, WallPos2, WallRect, WallResize, WallType};
use crate::grid::plugin::ComponentIDs;
use crate::math::transformations::{screen_to_grid, screen_to_nearest_grid};
use crate::ui::state::{ClipboardBuffer, ToolType, UiState};

pub fn copy_paste_system(
    keys: Res<ButtonInput<KeyCode>>,
    selected: Query<Entity, With<Selected>>,
    mut clipboard: ResMut<ClipboardBuffer>,
    mut ids: ResMut<ComponentIDs>,
    mut commands: Commands,
    sources: Query<(Entity, &Source), With<Selected>>,
    walls: Query<(Entity, &Wall), With<Selected>>,
    mics: Query<(Entity, &Microphone), With<Selected>>,
) {
    #[cfg(not(target_os = "macos"))]
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    #[cfg(target_os = "macos")]
    let ctrl = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);

    if ctrl && keys.just_pressed(KeyCode::KeyC) {
        if let Some(entity) = selected.iter().next() {
            clipboard.copy(entity);
        }
    }

    if ctrl && keys.just_pressed(KeyCode::KeyV) {
        if let Some(entity) = clipboard.get() {
            if let Ok((_, source)) = sources.get(entity) {
                let mut source = source.clone();
                source.id = ids.get_new_source_id();
                commands.spawn(source);
            } else if let Ok((_, wall)) = walls.get(entity) {
                let mut wall = wall.clone();
                wall.id = ids.get_new_wall_id();
                commands.spawn(wall);
            } else if let Ok((_, mic)) = mics.get(entity) {
                let mut mic = mic.clone();
                mic.id = ids.get_new_mic_id();
                commands.spawn(mic);
            }
        }
    }
}

pub fn button_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    sources: Query<(Entity, &Source), Without<Drag>>,
    mut drag_sources: Query<(Entity, &mut Source), With<Drag>>,
    microphones: Query<(Entity, &Microphone), Without<Drag>>,
    mut drag_microphones: Query<(Entity, &mut Microphone), With<Drag>>,
    mut selected: Query<Entity, With<Selected>>,
    walls: Query<(Entity, &Wall), (Without<Drag>, Without<WallResize>)>,
    mut drag_walls: Query<(Entity, &mut Wall), With<Drag>>,
    mut resize_walls: Query<(Entity, &WallResize, &mut Wall), (With<WallResize>, Without<Drag>)>,
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    mut component_ids: ResMut<ComponentIDs>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) && ui_state.tools_enabled {
        let window = q_windows.single();
        selected.iter_mut().for_each(|entity| {
            commands.entity(entity).remove::<Selected>();
        });
        if let Some(position) = window.cursor_position() {
            match ui_state.current_tool {
                ToolType::MoveSource => {
                    if let Some((x, y)) =
                        screen_to_nearest_grid(position.x, position.y, ui_state.image_rect)
                    {
                        for (entity, source) in sources.iter() {
                            let (s_x, s_y) = (source.x, source.y);
                            if s_x.abs_diff(x) <= 10 && s_y.abs_diff(y) <= 10 {
                                //values should change depending on image size (smaller image -> greater radius)
                                commands.entity(entity).insert((Drag, Selected));
                                break; // only drag one at a time
                            }
                        }
                    }
                }
                ToolType::PlaceSource => {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                    {
                        // this produces overlaping sources
                        commands.spawn(Source::new(
                            x,
                            y,
                            10.,
                            0.0,
                            10_000.0,
                            SourceType::Sin,
                            component_ids.get_new_source_id(),
                        ));
                    }
                }
                ToolType::DrawWall => match ui_state.wall_type {
                    WallType::Rectangle => {
                        if let Some((mut x, mut y)) =
                            screen_to_nearest_grid(position.x, position.y, ui_state.image_rect)
                        {
                            if keys.pressed(KeyCode::ControlLeft) {
                                x = (x as f32 / 10.).round() as u32 * 10;
                                y = (y as f32 / 10.).round() as u32 * 10;
                            }
                            commands.spawn((
                                Wall::new(
                                    WallType::Rectangle,
                                    ui_state.wall_hollowed,
                                    WallRect {
                                        min: WallPos2 { x, y },
                                        max: WallPos2 { x, y },
                                    },
                                    ui_state.wall_reflection_factor,
                                    component_ids.get_new_wall_id(),
                                ),
                                WallResize::BottomRight,
                                Overlay,
                            ));
                        }
                    }
                    WallType::Circle => {
                        if let Some((x, y)) =
                            screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                        {
                            commands.spawn((
                                Wall::new(
                                    WallType::Circle,
                                    ui_state.wall_hollowed,
                                    WallRect {
                                        min: WallPos2 { x, y },
                                        max: WallPos2 { x, y },
                                    },
                                    ui_state.wall_reflection_factor,
                                    component_ids.get_new_wall_id(),
                                ),
                                WallResize::Radius,
                                Overlay,
                            ));
                        }
                    }
                },
                ToolType::MoveWall => {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                    {
                        for (entity, wall) in walls.iter() {
                            let center = wall.get_center();
                            if (center.x).abs_diff(x) <= 10 && (center.y).abs_diff(y) <= 10 {
                                commands.entity(entity).insert((Drag, Selected));
                                break;
                            }
                        }
                    }
                }
                ToolType::ResizeWall => {
                    if let Some((x, y)) =
                        screen_to_nearest_grid(position.x, position.y, ui_state.image_rect)
                    {
                        for (entity, wall) in walls.iter() {
                            let resize_point = wall.get_resize_point();
                            if (resize_point.x).abs_diff(x) <= 10
                                && (resize_point.y).abs_diff(y) <= 10
                            {
                                commands
                                    .entity(entity)
                                    .insert((WallResize::BottomRight, Overlay));
                                break;
                            }
                        }
                    }
                }
                ToolType::PlaceMic => {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                    {
                        commands.spawn(Microphone::new(x, y, component_ids.get_new_mic_id()));
                    }
                }
                ToolType::MoveMic => {
                    if let Some((x, y)) =
                        screen_to_nearest_grid(position.x, position.y, ui_state.image_rect)
                    {
                        for (entity, mic) in microphones.iter() {
                            let (m_x, m_y) = (mic.x, mic.y);
                            if m_x.abs_diff(x) <= 10 && m_y.abs_diff(y) <= 10 {
                                //values should change depending on image size (smaller image -> greater radius)
                                commands.entity(entity).insert((Drag, Selected));
                                break; // only drag one at a time
                            }
                        }
                    }
                }
            }
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        drag_sources.iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<Drag>();
        });
        drag_microphones.iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<Drag>();
        });
        resize_walls.iter_mut().for_each(|(entity, _, wall)| {
            if wall.is_empty() {
                commands.entity(entity).despawn();
                component_ids.decrement_wall_ids();
            }
            commands.entity(entity).remove::<(WallResize, Overlay)>();
        });
        drag_walls.iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<Drag>();
        });
    }

    if mouse_buttons.pressed(MouseButton::Left) && ui_state.tools_enabled {
        let window = q_windows.single();

        if let Some(position) = window.cursor_position() {
            match ui_state.current_tool {
                ToolType::MoveSource => {
                    if let Some((x, y)) =
                        screen_to_nearest_grid(position.x, position.y, ui_state.image_rect)
                    {
                        drag_sources.iter_mut().for_each(|(_, mut source)| {
                            source.x = x;
                            source.y = y;
                        });
                    }
                }
                ToolType::DrawWall | ToolType::ResizeWall => {
                    if let Some((mut x, mut y)) =
                        screen_to_nearest_grid(position.x, position.y, ui_state.image_rect)
                    {
                        if keys.pressed(KeyCode::ControlLeft) {
                            x = (x as f32 / 10.).round() as u32 * 10;
                            y = (y as f32 / 10.).round() as u32 * 10;
                        }

                        resize_walls
                            .iter_mut()
                            .for_each(|(_, wall_resize, mut wall)| match wall_resize {
                                WallResize::BottomRight => {
                                    wall.rect.max.x = x;
                                    wall.rect.max.y = y;
                                    wall.update_calc_rect(ui_state.e_al);
                                }
                                WallResize::Radius => todo!(),
                                _ => {}
                            });
                    }
                }
                ToolType::MoveWall => {
                    if let Some((mut x, mut y)) =
                        screen_to_nearest_grid(position.x, position.y, ui_state.image_rect)
                    {
                        if keys.pressed(KeyCode::ControlLeft) {
                            x = (x as f32 / 10.).round() as u32 * 10;
                            y = (y as f32 / 10.).round() as u32 * 10;
                        }
                        drag_walls.iter_mut().for_each(|(_, mut wall)| {
                            wall.translate_center_to(x, y, ui_state.e_al);
                        });
                    }
                }
                ToolType::MoveMic => {
                    if let Some((x, y)) =
                        screen_to_nearest_grid(position.x, position.y, ui_state.image_rect)
                    {
                        drag_microphones.iter_mut().for_each(|(_, mut mic)| {
                            mic.x = x;
                            mic.y = y;
                        });
                    }
                }
                _ => {}
            }
        }
    }

    if keys.just_pressed(KeyCode::Space) {
        ui_state.is_running = !ui_state.is_running;
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        selected.iter_mut().for_each(|entity| {
            commands.entity(entity).despawn();
        });
    }
}
