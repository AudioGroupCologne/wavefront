use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_pixel_buffer::bevy_egui::egui::Pos2;

use crate::components::source::{Drag, Source, SourceType};
use crate::components::wall::{CornerResize, WallBlock, WallCell};
use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::transformations::{screen_to_grid, screen_to_nearest_grid};
use crate::render::state::{ToolType, UiState, WallBrush};

pub fn button_input(
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    sources: Query<(Entity, &Source), Without<Drag>>,
    mut drag_sources: Query<(Entity, &mut Source), With<Drag>>,
    wallblocks: Query<(Entity, &WallBlock), (Without<Drag>, Without<CornerResize>)>,
    mut drag_wallblocks: Query<(Entity, &mut WallBlock), With<Drag>>,
    mut resize_wallblocks: Query<(Entity, &mut WallBlock), (With<CornerResize>, Without<Drag>)>,
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let window = q_windows.single();
        match ui_state.current_tool {
            ToolType::MoveSource => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) = screen_to_nearest_grid(
                        position.x,
                        position.y,
                        ui_state.image_rect,
                        &ui_state,
                    ) {
                        for (entity, source) in sources.iter() {
                            let (s_x, s_y) = (source.x, source.y);
                            if s_x.abs_diff(x) <= 10 && s_y.abs_diff(y) <= 10 {
                                //values should change depending on image size (smaller image -> greater radius)
                                commands.entity(entity).insert(Drag);
                                break; // only drag one at a time
                            }
                        }
                    }
                }
            }
            ToolType::PlaceSource => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                    {
                        // this produces overlaping sources
                        commands.spawn(Source::new(x, y, 10., 0.0, 10_000.0, SourceType::Sin));
                    }
                }
            }
            ToolType::DrawWall => match ui_state.wall_brush {
                WallBrush::Rectangle => {
                    if let Some(position) = window.cursor_position() {
                        if let Some((x, y)) = screen_to_nearest_grid(
                            position.x,
                            position.y,
                            ui_state.image_rect,
                            &ui_state,
                        ) {
                            commands.spawn((
                                WallBlock::new(x, y, ui_state.wall_reflection_factor),
                                CornerResize,
                            ));
                        }
                    }
                }
                WallBrush::CircleBrush => {
                    if let Some(position) = window.cursor_position() {
                        if let Some((x, y)) =
                            screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                        {
                            for dx in -(ui_state.wall_brush_radius as i32)
                                ..ui_state.wall_brush_radius as i32
                            {
                                for dy in -(ui_state.wall_brush_radius as i32)
                                    ..ui_state.wall_brush_radius as i32
                                {
                                    if dx * dx + dy * dy
                                        <= ui_state.wall_brush_radius as i32
                                            * ui_state.wall_brush_radius as i32
                                    {
                                        let x = (x as i32 + dx)
                                            .clamp(0, SIMULATION_WIDTH as i32 - 1)
                                            as u32;
                                        let y = (y as i32 + dy)
                                            .clamp(0, SIMULATION_HEIGHT as i32 - 1)
                                            as u32;
                                        commands.spawn((WallCell::new(
                                            x,
                                            y,
                                            ui_state.wall_reflection_factor,
                                        ),));
                                    }
                                }
                            }
                        }
                    }
                }
            },
            ToolType::MoveWall => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                    {
                        for (entity, wall) in wallblocks.iter() {
                            let center = wall.rect.center();
                            if (center.x as u32).abs_diff(x) <= 10
                                && (center.y as u32).abs_diff(y) <= 10
                            {
                                commands.entity(entity).insert(Drag);
                                break;
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
        resize_wallblocks.iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<CornerResize>();
        });
        drag_wallblocks.iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<Drag>();
        });
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        let window = q_windows.single();

        match ui_state.current_tool {
            ToolType::MoveSource => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) = screen_to_nearest_grid(
                        position.x,
                        position.y,
                        ui_state.image_rect,
                        &ui_state,
                    ) {
                        drag_sources.iter_mut().for_each(|(_, mut source)| {
                            source.x = x;
                            source.y = y;
                        });
                    }
                }
            }
            ToolType::DrawWall => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) = screen_to_nearest_grid(
                        position.x,
                        position.y,
                        ui_state.image_rect,
                        &ui_state,
                    ) {
                        resize_wallblocks.iter_mut().for_each(|(_, mut wall)| {
                            wall.rect.max.x = x as f32;
                            wall.rect.max.y = y as f32;
                            wall.update_calc_rect();
                        });
                    }
                }
            }
            ToolType::MoveWall => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) = screen_to_nearest_grid(
                        position.x,
                        position.y,
                        ui_state.image_rect,
                        &ui_state,
                    ) {
                        drag_wallblocks.iter_mut().for_each(|(_, mut wall)| {
                            let width = wall.rect.width() / 2.;
                            let height = wall.rect.height() / 2.;
                            wall.rect.min = Pos2::new(x as f32 - width, y as f32 - height);
                            wall.rect.max = Pos2::new(x as f32 + width, y as f32 + height);
                            wall.update_calc_rect();
                        });
                    }
                }
            }
            _ => {}
        }
    }

    // if mouse_buttons.just_pressed(MouseButton::Right) {
    //
    // }

    // if mouse_buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
    // }

    if keys.just_pressed(KeyCode::Space) {
        ui_state.is_running = !ui_state.is_running;
    }
}
