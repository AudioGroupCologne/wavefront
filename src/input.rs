use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_pixel_buffer::bevy_egui::egui::emath::Rect;

use crate::components::{u32_map_range, Drag, Source, SourceType};
use crate::constants::*;
use crate::render::{ToolType, UiState};

fn screen_to_grid(x: f32, y: f32, image_rect: Rect) -> Option<(u32, u32)> {
    let width = image_rect.width();
    let height = image_rect.height();
    let x = x - image_rect.min.x;
    let y = y - image_rect.min.y;

    if x >= width || y >= height || x < 0. || y < 0. {
        return None;
    }

    Some((
        u32_map_range(0, width as u32, 0, SIMULATION_WIDTH, x as u32),
        u32_map_range(0, height as u32, 0, SIMULATION_HEIGHT, y as u32),
    ))
}

pub fn button_input(
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    sources: Query<(Entity, &Source), Without<Drag>>,
    mut drag_sources: Query<(Entity, &mut Source), With<Drag>>,
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let window = q_windows.single();
        match ui_state.tool_type {
            ToolType::MoveSource => {
                if let Some(position) = window.cursor_position() {
                    if let Some((x, y)) =
                        screen_to_grid(position.x, position.y, ui_state.image_rect)
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
                        screen_to_grid(position.x, position.y, ui_state.image_rect)
                    {
                        // this produces overlaping sources
                        commands.spawn(Source::new(x, y, 10., 0.0, 10_000.0, SourceType::Sin));
                    }
                }
            }
        }
    }
    if mouse_buttons.just_released(MouseButton::Left) {
        drag_sources.iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<Drag>();
        });
    }
    if mouse_buttons.pressed(MouseButton::Left) && drag_sources.iter_mut().count() >= 1 {
        let window = q_windows.single();
        if let Some(position) = window.cursor_position() {
            if let Some((x, y)) = screen_to_grid(position.x, position.y, ui_state.image_rect) {
                drag_sources.iter_mut().for_each(|(_, mut source)| {
                    source.x = x;
                    source.y = y;
                });
            }
        }
    }
    // if mouse_buttons.just_pressed(MouseButton::Right) {
    //     let window = q_windows.single();
    //     if let Some(position) = window.cursor_position() {
    //         if let Some((x, y)) = screen_to_grid(position.x, position.y, ui_state.image_rect.min) {
    //             // this produces overlaping sources
    //             commands.spawn(Source::new(x, y, 10., 0.0, 10_000.0, SourceType::Sin));

    //             // //TODO: because of the brush size, the indices may be out of bounds
    //             // //TODO: make bush size variable
    //             // commands.spawn(Wall(Grid::coords_to_index(x, y, 0)));
    //             // commands.spawn(Wall(Grid::coords_to_index(x + 1, y, 0)));
    //             // commands.spawn(Wall(Grid::coords_to_index(x - 1, y, 0)));
    //             // commands.spawn(Wall(Grid::coords_to_index(x, y + 1, 0)));
    //             // commands.spawn(Wall(Grid::coords_to_index(x + 1, y + 1, 0)));
    //             // commands.spawn(Wall(Grid::coords_to_index(x, y - 1, 0)));
    //             // commands.spawn(Wall(Grid::coords_to_index(x - 1, y - 1, 0)));
    //             // commands.spawn(Wall(Grid::coords_to_index(x + 1, y - 1, 0)));
    //             // commands.spawn(Wall(Grid::coords_to_index(x - 1, y + 1, 0)));
    //         }
    //     }
    // }

    // if mouse_buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
    // }

    if keys.just_pressed(KeyCode::Space) {
        ui_state.is_running = !ui_state.is_running;
    }
}
