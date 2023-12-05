use bevy::{prelude::*, window::PrimaryWindow};
use bevy_pixel_buffer::bevy_egui::egui::Pos2;

use crate::components::{Drag, Source, SourceType, Wall};
use crate::constants::*;
use crate::grid::Grid;

use crate::render::UiState;

fn screen_to_grid(x: f32, y: f32, image_rect_top: Pos2) -> Option<(u32, u32)> {
    let x = (x - image_rect_top.x) as i32;
    let y = (y - image_rect_top.y) as i32;

    if x >= SIMULATION_WIDTH as i32 || y >= SIMULATION_HEIGHT as i32 || x < 0 || y < 0 {
        return None;
    }

    Some((x as u32, y as u32))
}

pub fn mouse_button_input(
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
        if let Some(position) = window.cursor_position() {
            if let Some((x, y)) = screen_to_grid(position.x, position.y, ui_state.image_rect_top) {
                for (entity, source) in sources.iter() {
                    let (s_x, s_y) = (source.x, source.y);
                    if s_x.abs_diff(x) <= 10 && s_y.abs_diff(y) <= 10 {
                        commands.entity(entity).insert(Drag);
                        break; // only drag one at a time
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
            if let Some((x, y)) = screen_to_grid(position.x, position.y, ui_state.image_rect_top) {
                drag_sources.iter_mut().for_each(|(_, mut source)| {
                    source.x = x;
                    source.y = y;
                });
            }
        }
    }
    if mouse_buttons.just_pressed(MouseButton::Right) {
        let window = q_windows.single();
        if let Some(position) = window.cursor_position() {
            if let Some((x, y)) = screen_to_grid(position.x, position.y, ui_state.image_rect_top) {
                // this produces overlaping sources
                commands.spawn(Source::new(x, y, 10., 0.0, 10_000.0, SourceType::Sin));

                // //TODO: because of the brush size, the indices may be out of bounds
                // //TODO: make bush size variable
                // commands.spawn(Wall(Grid::coords_to_index(x, y, 0)));
                // commands.spawn(Wall(Grid::coords_to_index(x + 1, y, 0)));
                // commands.spawn(Wall(Grid::coords_to_index(x - 1, y, 0)));
                // commands.spawn(Wall(Grid::coords_to_index(x, y + 1, 0)));
                // commands.spawn(Wall(Grid::coords_to_index(x + 1, y + 1, 0)));
                // commands.spawn(Wall(Grid::coords_to_index(x, y - 1, 0)));
                // commands.spawn(Wall(Grid::coords_to_index(x - 1, y - 1, 0)));
                // commands.spawn(Wall(Grid::coords_to_index(x + 1, y - 1, 0)));
                // commands.spawn(Wall(Grid::coords_to_index(x - 1, y + 1, 0)));
            }
        }
    }
    // we can check multiple at once with `.any_*`
    if mouse_buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        // Either the left or the right button was just pressed
    }

    if keys.just_pressed(KeyCode::Space) {
        ui_state.is_running = !ui_state.is_running;
    }
}
