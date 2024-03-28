use bevy::prelude::*;

use super::grid::Grid;
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::ui::state::{SimTime, UiState};

pub fn calc_system(mut grid: ResMut<Grid>, ui_state: Res<UiState>) {
    if ui_state.is_running {
        grid.calc_cells(ui_state.boundary_width);
    }
}

pub fn apply_system(
    mut grid: ResMut<Grid>,
    sources: Query<&Source>,
    microphones: Query<&mut Microphone>,
    sim_time: Res<SimTime>,
    ui_state: Res<UiState>,
) {
    if ui_state.is_running {
        grid.apply_sources(sim_time.time_since_start, &sources, ui_state.boundary_width);
        grid.apply_microphones(microphones, &ui_state);
    }
}

pub fn update_system(
    mut grid: ResMut<Grid>,
    mut sim_time: ResMut<SimTime>,
    ui_state: Res<UiState>,
) {
    if ui_state.is_running {
        grid.update_cells();
        grid.update_delta_t(ui_state.delta_l);
        sim_time.time_since_start += grid.delta_t;
    }
}
