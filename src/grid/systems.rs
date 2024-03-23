use bevy::prelude::*;

use super::grid::Grid;
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::ui::state::{GameTicks, UiState};

pub fn calc_system(mut grid: ResMut<Grid>, ui_state: Res<UiState>) {
    if ui_state.is_running {
        // grid.apply_boundaries(ui_state.boundary_width);
        grid.calc_cells(ui_state.boundary_width);
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
        grid.update_delta_t(ui_state.delta_l);
        game_ticks.ticks_since_start += 1;
    }
}
