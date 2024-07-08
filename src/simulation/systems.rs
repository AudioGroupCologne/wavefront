use bevy::prelude::*;

use super::grid::Grid;
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::ui::state::{SimTime, UiState};

/// A system used to calculate reflection pulses per cell
pub fn calc_system(mut grid: ResMut<Grid>, ui_state: Res<UiState>) {
    if ui_state.is_running {
        grid.calc_cells(ui_state.boundary_width);
    }
}

/// A system used to insert source reflection pulses into cells
///
/// and write pressure values into microphones
pub fn apply_system(
    mut grid: ResMut<Grid>,
    sources: Query<&Source>,
    microphones: Query<&mut Microphone>,
    sim_time: Res<SimTime>,
    ui_state: Res<UiState>,
) {
    if ui_state.is_running {
        grid.apply_sources(sim_time.time_since_start, &sources, ui_state.boundary_width);
        grid.apply_microphones(microphones, &ui_state, sim_time.time_since_start as f64);
    }
}

/// A system used to write reflection pulses into the incident pulses,
///
/// update delta t
///
/// and update the simulation time
pub fn update_system(
    mut grid: ResMut<Grid>,
    mut sim_time: ResMut<SimTime>,
    ui_state: Res<UiState>,
) {
    if ui_state.is_running {
        grid.update_cells();
        sim_time.time_since_start += grid.delta_t;
    }
}
