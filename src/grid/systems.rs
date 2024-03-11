use bevy::prelude::*;

use super::grid::Grid;
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::states::Overlay;
use crate::components::wall::Wall;
use crate::ui::state::{GameTicks, UiState};

pub fn calc_system(
    mut grid: ResMut<Grid>,
    walls: Query<&Wall, Without<Overlay>>,
    ui_state: Res<UiState>,
) {
    if ui_state.is_running {
        grid.calc_cells(&walls, ui_state.boundary_width);
    }
}

pub fn apply_system(
    mut grid: ResMut<Grid>,
    sources: Query<&Source>,
    microphones: Query<&mut Microphone>,
    game_ticks: Res<GameTicks>,
    ui_state: Res<UiState>,
) {
    if ui_state.is_running {
        grid.apply_sources(
            game_ticks.ticks_since_start,
            &sources,
            ui_state.boundary_width,
        );
        grid.apply_boundaries(&ui_state);
        grid.apply_microphones(microphones, &ui_state);
    }
}

pub fn update_system(
    mut grid: ResMut<Grid>,
    mut game_ticks: ResMut<GameTicks>,
    ui_state: Res<UiState>,
) {
    if ui_state.is_running {
        grid.update_cells();
        grid.update_delta_t(&ui_state);
        game_ticks.ticks_since_start += 1;
    }
}
