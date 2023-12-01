use bevy::ui;
use bevy::{prelude::*, window::PrimaryWindow};

use crate::components::{Drag, Source, SourceType, Wall};
use crate::constants::*;
use crate::grid::Grid;

use crate::render::UiState;

fn screen_to_grid(x: f32, y: f32, screen_width: f32, screen_height: f32) -> Option<(u32, u32)> {
    let x = (x - (screen_width - (SIMULATION_WIDTH / PIXEL_SIZE) as f32) / 2.) as u32;
    let y = (y - (screen_height - (SIMULATION_HEIGHT / PIXEL_SIZE) as f32) / 2.) as u32;

    if x >= SIMULATION_WIDTH || x <= 0 || y <= 0 || y >= SIMULATION_HEIGHT {
        return None;
    }

    Some((x, y))
}

pub fn mouse_button_input(
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    sources: Query<(Entity, &Source), Without<Drag>>,
    mut drag_sources: Query<(Entity, &mut Source), With<Drag>>,
    mut commands: Commands,
    ui_state: Res<UiState>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let window = q_windows.single();
        if let Some(position) = window.cursor_position() {
            if let Some((x, y)) =
                screen_to_grid(position.x, position.y, window.width(), window.height())
            {
                // grid.sources.push(Source::new(
                //     array_pos(x, y, 0),
                //     10.,
                //     0.0,
                //     1.0,
                //     SourceType::Sin,
                // ));
                for (entity, source) in sources.iter() {
                    let (s_x, s_y) = Grid::index_to_coords(source.index as u32, ui_state.e_al);
                    if s_x.abs_diff(x) <= 10 && s_y.abs_diff(y) <= 10 {
                        commands.entity(entity).insert(Drag);
                    }
                }
            }
        }
    }
    if buttons.just_released(MouseButton::Left) {
        drag_sources.iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<Drag>();
        });
    }
    if buttons.pressed(MouseButton::Left) && drag_sources.iter_mut().count() >= 1 {
        let window = q_windows.single();
        if let Some(position) = window.cursor_position() {
            if let Some((x, y)) =
                screen_to_grid(position.x, position.y, window.width(), window.height())
            {
                drag_sources.iter_mut().for_each(|(_, mut source)| {
                    source.index = Grid::coords_to_index(x, y, 0, ui_state.e_al);
                });
            }
        }
    }
    if buttons.pressed(MouseButton::Right) {
        let window = q_windows.single();
        if let Some(position) = window.cursor_position() {
            if let Some((x, y)) =
                screen_to_grid(position.x, position.y, window.width(), window.height())
            {
                // this produces overlaping sources
                commands.spawn(Source::new(
                    Grid::coords_to_index(x, y, 0, ui_state.e_al),
                    x,
                    y,
                    10.,
                    0.0,
                    10_000.0,
                    SourceType::Sin,
                ));

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
    if buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        // Either the left or the right button was just pressed
    }
}
