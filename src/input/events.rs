use bevy::ecs::event::{Event, EventReader};
use bevy::ecs::system::{Query, Res, ResMut};

use crate::components::wall::{CircWall, RectWall};
use crate::grid::grid::Grid;
use crate::ui::state::UiState;

#[derive(Event)]
pub struct UpdateWalls;

// fn player_level_up(mut ev_levelup: EventWriter<LevelUpEvent>, query: Query<(Entity, &PlayerXp)>) {
//     for (entity, xp) in query.iter() {
//         if xp.0 > 1000 {
//             ev_levelup.send(LevelUpEvent(entity));
//         }
//     }
// }

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
