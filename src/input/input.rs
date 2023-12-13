use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::source::{Drag, Source, SourceType};
use crate::components::wall::{CornerResize, WallBlock};
use crate::math::transformations::screen_to_grid;
use crate::render::state::{ToolType, UiState};

pub fn button_input(
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    sources: Query<(Entity, &Source), Without<Drag>>,
    walls: Query<(Entity, &Source), Without<Drag>>,
    mut drag_sources: Query<(Entity, &mut Source), With<Drag>>,
    mut resize_walls: Query<(Entity, &mut WallBlock), With<CornerResize>>,
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let window = q_windows.single();
        match ui_state.current_tool {
            ToolType::MoveSource => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                    {
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
            ToolType::DrawWall => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                    {
                        //TODO: because of the brush size, the indices may be out of bounds
                        //TODO: make bush size variable
                        //TODO: make wall act on x and y coords like sources
                        commands.spawn((WallBlock::new(x, y, 1.), CornerResize));
                    }
                }
            }
            ToolType::MoveWall => {
                // if let Some(position) = window.cursor_position() {
                //     if let Some((x, y)) =
                //         screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                //     {
                //         for (entity, wall) in walls.iter() {
                //             let (w_x, w_y) = (source.x, source.y);
                //             if s_x.abs_diff(x) <= 10 && s_y.abs_diff(y) <= 10 {
                //                 //values should change depending on image size (smaller image -> greater radius)
                //                 commands.entity(entity).insert(Drag);
                //                 break; // only drag one at a time
                //             }
                //         }
                //     }
                // }
            }
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        drag_sources.iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<Drag>();
        });
        resize_walls.iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<CornerResize>();
        });
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        let window = q_windows.single();

        match ui_state.current_tool {
            ToolType::MoveSource => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                    {
                        drag_sources.iter_mut().for_each(|(_, mut source)| {
                            source.x = x;
                            source.y = y;
                        });
                    }
                }
            }
            ToolType::DrawWall => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect, &ui_state)
                    {
                        resize_walls.iter_mut().for_each(|(_, mut wall)| {
                            wall.rect.max.x = x as f32;
                            wall.rect.max.y = y as f32;
                            wall.update();
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
