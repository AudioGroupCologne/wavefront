use bevy::prelude::*;

use crate::components::wall::{CircWall, RectWall};
use crate::grid::grid::Grid;
use crate::ui::state::UiState;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (update_wall_event, reset_event))
            .add_event::<UpdateWalls>()
            .add_event::<Reset>();
    }
}

#[derive(Event)]
pub struct UpdateWalls;

pub fn update_wall_event(
    mut wall_update_ev: EventReader<UpdateWalls>,
    mut grid: ResMut<Grid>,
    ui_state: Res<UiState>,
    rect_walls: Query<&RectWall>,
    circ_walls: Query<&CircWall>,
) {
    for _ in wall_update_ev.read() {
        grid.update_walls(&rect_walls, &circ_walls, ui_state.boundary_width);
    }
}

#[derive(Event)]
pub struct Reset;

pub fn reset_event(
    mut reset_ev: EventReader<Reset>,
    mut grid: ResMut<Grid>,
    ui_state: Res<UiState>,
) {
    if ui_state.reset_on_change {
        for _ in reset_ev.read() {
            grid.reset_cells(ui_state.boundary_width);
        }
    }
}
