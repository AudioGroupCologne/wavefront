use bevy::prelude::*;

use super::grid::Grid;
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::WallBlock;
use crate::render::state::{GameTicks, UiState};

pub fn calc_system(mut grid: ResMut<Grid>, ui_state: Res<UiState>) {
    if ui_state.is_running {
        grid.calc(ui_state.e_al);
    }
}

pub fn apply_system(
    mut grid: ResMut<Grid>,
    sources: Query<&Source>,
    microphones: Query<&mut Microphone>,
    walls: Query<&WallBlock>,
    game_ticks: Res<GameTicks>,
    ui_state: Res<UiState>,
) {
    if ui_state.is_running {
        grid.apply_sources(game_ticks.ticks_since_start, &sources, ui_state.e_al);
        grid.apply_walls(&walls, ui_state.e_al);
        grid.apply_microphones(microphones, &ui_state);
        grid.apply_boundaries(ui_state);
    }
}

pub fn update_system(
    mut grid: ResMut<Grid>,
    mut game_ticks: ResMut<GameTicks>,
    ui_state: Res<UiState>,
) {
    if ui_state.is_running {
        grid.update(ui_state.e_al);
        grid.update_delta_t(ui_state);
        game_ticks.ticks_since_start += 1;
    }
}
