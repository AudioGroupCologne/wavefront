use std::f32::consts::PI;

use bevy::prelude::*;

use crate::components::{GameTicks, Microphone, Source, SourceType, Wall};
use crate::constants::*;
use crate::render::{AttenuationType, UiState};

#[derive(Debug, Resource)]
pub struct Grid {
    /// full grid: [cur_bottom, cur_left, cur_top, cur_right, next_bottom, next_left, next_top, next_right, pressure]
    pub cells: Vec<f32>,
    /// Delta L in meters
    pub delta_l: f32,
    /// Delta s in seconds
    pub delta_t: f32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cells: vec![
                0.;
                ((SIMULATION_WIDTH + 2 * E_AL) * (SIMULATION_HEIGHT + 2 * E_AL) * NUM_INDEX)
                    as usize
            ],
            delta_l: 0.001, //basically useless when using val from ui
            delta_t: 0.001 / PROPAGATION_SPEED,
        }
    }
}

impl Grid {
    fn update_delta_t(&mut self, ui_state: Res<UiState>) {
        self.delta_t = ui_state.delta_l / PROPAGATION_SPEED;
    }

    pub fn update_cells(&mut self, e_al: u32) {
        self.cells = vec![
            0.;
            ((SIMULATION_WIDTH + 2 * e_al) * (SIMULATION_HEIGHT + 2 * e_al) * NUM_INDEX)
                as usize
        ];
    }

    fn update(&mut self, e_al: u32) {
        for i in 0..(SIMULATION_WIDTH + 2 * e_al) * (SIMULATION_HEIGHT + 2 * e_al) {
            let array_pos: usize = (i * NUM_INDEX) as usize;

            self.cells[array_pos] = self.cells[array_pos + 4];
            self.cells[array_pos + 1] = self.cells[array_pos + 5];
            self.cells[array_pos + 2] = self.cells[array_pos + 6];
            self.cells[array_pos + 3] = self.cells[array_pos + 7];

            //calculate pressure
            self.cells[array_pos + 8] = 0.5
                * (self.cells[array_pos]
                    + self.cells[array_pos + 1]
                    + self.cells[array_pos + 2]
                    + self.cells[array_pos + 3]);
        }
    }

    fn calc(&mut self, e_al: u32) {
        for x in e_al..(SIMULATION_WIDTH + e_al) {
            for y in e_al..(SIMULATION_HEIGHT + e_al) {
                self.calc_cell(
                    Grid::coords_to_index(x, y, 0, e_al),
                    self.cells[Grid::coords_to_index(x, y + 1, 2, e_al)],
                    self.cells[Grid::coords_to_index(x - 1, y, 3, e_al)],
                    self.cells[Grid::coords_to_index(x, y - 1, 0, e_al)],
                    self.cells[Grid::coords_to_index(x + 1, y, 1, e_al)],
                );
            }
        }
    }

    fn calc_cell(
        &mut self,
        coord_one_d: usize,
        bottom_top: f32,
        left_right: f32,
        top_bottom: f32,
        right_left: f32,
    ) {
        self.cells[coord_one_d + 4] = 0.5 * (-bottom_top + left_right + top_bottom + right_left);
        self.cells[coord_one_d + 5] = 0.5 * (bottom_top - left_right + top_bottom + right_left);
        self.cells[coord_one_d + 6] = 0.5 * (bottom_top + left_right - top_bottom + right_left);
        self.cells[coord_one_d + 7] = 0.5 * (bottom_top + left_right + top_bottom - right_left);
    }

    fn apply_sources(&mut self, ticks_since_start: u64, sources: &Query<&Source>, e_al: u32) {
        let time = self.delta_t * ticks_since_start as f32; //the cast feels wrong, but it works for now
        for source in sources.iter() {
            //? maybe needs to be optimized
            let calc = match source.r#type {
                SourceType::Sin => {
                    source.amplitude
                        * (2. * PI * source.frequency * (time - source.phase * PI / 180.)).sin()
                }
                SourceType::Gauss => {
                    Source::periodic_gaussian(time, source.frequency, source.amplitude, 5., 1.)
                }
            };
            let source_pos = Grid::coords_to_index(source.x + e_al, source.y + e_al, 0, e_al); //source.index;
            self.cells[source_pos + 4] = calc;
            self.cells[source_pos + 5] = calc;
            self.cells[source_pos + 6] = calc;
            self.cells[source_pos + 7] = calc;
        }
    }

    fn apply_microphones(
        &mut self, //doesn't actually need to mutable but it throws errors further down if not
        ticks_since_start: u64,
        mut microphones: Query<&mut Microphone>,
        e_al: u32,
    ) {
        for mut mic in microphones.iter_mut() {
            let x = mic.x;
            let y = mic.y;
            mic.record.push([
                (ticks_since_start as f32 * self.delta_t) as f64,
                self.cells[Grid::coords_to_index(x, y, 8, e_al)] as f64,
            ]);
        }
    }

    fn apply_walls(&mut self, walls: &Query<&Wall>, e_al: u32) {
        for wall in walls.iter() {
            let (x, y) = Grid::index_to_coords(wall.0 as u32, e_al);
            self.cells[wall.0 + 4] =
                WALL_FAC * self.cells[Grid::coords_to_index(x, y + 1, 2, e_al)];
            self.cells[wall.0 + 5] =
                WALL_FAC * self.cells[Grid::coords_to_index(x - 1, y, 3, e_al)];
            self.cells[wall.0 + 6] =
                WALL_FAC * self.cells[Grid::coords_to_index(x, y - 1, 0, e_al)];
            self.cells[wall.0 + 7] =
                WALL_FAC * self.cells[Grid::coords_to_index(x + 1, y, 1, e_al)];
        }
    }

    fn apply_boundaries(&mut self, ui_state: Res<UiState>) {
        let b = (ui_state.e_al * ui_state.e_al) as f32 / (f32::ln(ui_state.epsilon));
        //Left
        for x in 1..ui_state.e_al {
            for y in ui_state.e_al..(SIMULATION_HEIGHT + ui_state.e_al) {
                let attenuation_factor = Grid::attenuation_factor(
                    ui_state.e_al,
                    ui_state.e_al - x,
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let current_cell = Grid::coords_to_index(x, y, 0, ui_state.e_al);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2, ui_state.e_al)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3, ui_state.e_al)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0, ui_state.e_al)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1, ui_state.e_al)];
                self.cells[current_cell + 4] =
                    0.5 * (-bottom_top + left_right + top_bottom + attenuation_factor * right_left);
                self.cells[current_cell + 5] =
                    0.5 * (bottom_top - left_right + top_bottom + attenuation_factor * right_left);
                self.cells[current_cell + 6] =
                    0.5 * (bottom_top + left_right - top_bottom + attenuation_factor * right_left);
                self.cells[current_cell + 7] =
                    0.5 * (bottom_top + left_right + top_bottom - attenuation_factor * right_left);
            }
        }
        //Top
        for x in ui_state.e_al..(SIMULATION_WIDTH + ui_state.e_al) {
            for y in 1..ui_state.e_al {
                let attenuation_factor = Grid::attenuation_factor(
                    ui_state.e_al,
                    ui_state.e_al - y,
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let current_cell = Grid::coords_to_index(x, y, 0, ui_state.e_al);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2, ui_state.e_al)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3, ui_state.e_al)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0, ui_state.e_al)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1, ui_state.e_al)];
                self.cells[current_cell + 4] =
                    0.5 * (-attenuation_factor * bottom_top + left_right + top_bottom + right_left);
                self.cells[current_cell + 5] =
                    0.5 * (attenuation_factor * bottom_top - left_right + top_bottom + right_left);
                self.cells[current_cell + 6] =
                    0.5 * (attenuation_factor * bottom_top + left_right - top_bottom + right_left);
                self.cells[current_cell + 7] =
                    0.5 * (attenuation_factor * bottom_top + left_right + top_bottom - right_left);
            }
        }
        //Right
        for x in (SIMULATION_WIDTH + ui_state.e_al)..(SIMULATION_WIDTH + 2 * ui_state.e_al - 1) {
            for y in ui_state.e_al..(SIMULATION_HEIGHT + ui_state.e_al) {
                let attenuation_factor = Grid::attenuation_factor(
                    ui_state.e_al,
                    x - (SIMULATION_WIDTH + ui_state.e_al),
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let current_cell = Grid::coords_to_index(x, y, 0, ui_state.e_al);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2, ui_state.e_al)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3, ui_state.e_al)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0, ui_state.e_al)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1, ui_state.e_al)];
                self.cells[current_cell + 4] =
                    0.5 * (-bottom_top + attenuation_factor * left_right + top_bottom + right_left);
                self.cells[current_cell + 5] =
                    0.5 * (bottom_top - attenuation_factor * left_right + top_bottom + right_left);
                self.cells[current_cell + 6] =
                    0.5 * (bottom_top + attenuation_factor * left_right - top_bottom + right_left);
                self.cells[current_cell + 7] =
                    0.5 * (bottom_top + attenuation_factor * left_right + top_bottom - right_left);
            }
        }
        //Bottom
        for x in ui_state.e_al..(SIMULATION_WIDTH + ui_state.e_al) {
            for y in
                (SIMULATION_HEIGHT + ui_state.e_al)..(SIMULATION_HEIGHT + 2 * ui_state.e_al - 1)
            {
                let attenuation_factor = Grid::attenuation_factor(
                    ui_state.e_al,
                    y - (SIMULATION_HEIGHT + ui_state.e_al),
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let current_cell = Grid::coords_to_index(x, y, 0, ui_state.e_al);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2, ui_state.e_al)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3, ui_state.e_al)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0, ui_state.e_al)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1, ui_state.e_al)];
                self.cells[current_cell + 4] =
                    0.5 * (-bottom_top + left_right + attenuation_factor * top_bottom + right_left);
                self.cells[current_cell + 5] =
                    0.5 * (bottom_top - left_right + attenuation_factor * top_bottom + right_left);
                self.cells[current_cell + 6] =
                    0.5 * (bottom_top + left_right - attenuation_factor * top_bottom + right_left);
                self.cells[current_cell + 7] =
                    0.5 * (bottom_top + left_right + attenuation_factor * top_bottom - right_left);
            }
        }
        //LeftTop
        for x in 1..ui_state.e_al {
            for y in 1..ui_state.e_al {
                let attenuation_factor_left = Grid::attenuation_factor(
                    ui_state.e_al,
                    ui_state.e_al - x,
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let attenuation_factor_top = Grid::attenuation_factor(
                    ui_state.e_al,
                    ui_state.e_al - y,
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let current_cell = Grid::coords_to_index(x, y, 0, ui_state.e_al);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2, ui_state.e_al)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3, ui_state.e_al)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0, ui_state.e_al)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1, ui_state.e_al)];
                self.cells[current_cell + 4] = 0.5
                    * (-attenuation_factor_top * bottom_top
                        + left_right
                        + top_bottom
                        + attenuation_factor_left * right_left);
                self.cells[current_cell + 5] = 0.5
                    * (attenuation_factor_top * bottom_top - left_right
                        + top_bottom
                        + attenuation_factor_left * right_left);
                self.cells[current_cell + 6] = 0.5
                    * (attenuation_factor_top * bottom_top + left_right - top_bottom
                        + attenuation_factor_left * right_left);
                self.cells[current_cell + 7] = 0.5
                    * (attenuation_factor_top * bottom_top + left_right + top_bottom
                        - attenuation_factor_left * right_left);
            }
        }
        //RightTop
        for x in (SIMULATION_WIDTH + ui_state.e_al)..(SIMULATION_WIDTH + 2 * ui_state.e_al - 1) {
            for y in 1..ui_state.e_al {
                let attenuation_factor_right = Grid::attenuation_factor(
                    ui_state.e_al,
                    x - (SIMULATION_WIDTH + ui_state.e_al),
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let attenuation_factor_top = Grid::attenuation_factor(
                    ui_state.e_al,
                    ui_state.e_al - y,
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let current_cell = Grid::coords_to_index(x, y, 0, ui_state.e_al);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2, ui_state.e_al)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3, ui_state.e_al)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0, ui_state.e_al)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1, ui_state.e_al)];
                self.cells[current_cell + 4] = 0.5
                    * (-attenuation_factor_top * bottom_top
                        + attenuation_factor_right * left_right
                        + top_bottom
                        + right_left);
                self.cells[current_cell + 5] = 0.5
                    * (attenuation_factor_top * bottom_top - attenuation_factor_right * left_right
                        + top_bottom
                        + right_left);
                self.cells[current_cell + 6] = 0.5
                    * (attenuation_factor_top * bottom_top + attenuation_factor_right * left_right
                        - top_bottom
                        + right_left);
                self.cells[current_cell + 7] = 0.5
                    * (attenuation_factor_top * bottom_top
                        + attenuation_factor_right * left_right
                        + top_bottom
                        - right_left);
            }
        }
        //RightBottom
        for x in (SIMULATION_WIDTH + ui_state.e_al)..(SIMULATION_WIDTH + 2 * ui_state.e_al - 1) {
            for y in
                (SIMULATION_HEIGHT + ui_state.e_al)..(SIMULATION_HEIGHT + 2 * ui_state.e_al - 1)
            {
                let attenuation_factor_right = Grid::attenuation_factor(
                    ui_state.e_al,
                    x - (SIMULATION_WIDTH + ui_state.e_al),
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let attenuation_factor_bottom = Grid::attenuation_factor(
                    ui_state.e_al,
                    y - (SIMULATION_HEIGHT + ui_state.e_al),
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let current_cell = Grid::coords_to_index(x, y, 0, ui_state.e_al);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2, ui_state.e_al)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3, ui_state.e_al)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0, ui_state.e_al)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1, ui_state.e_al)];
                self.cells[current_cell + 4] = 0.5
                    * (-bottom_top
                        + attenuation_factor_right * left_right
                        + attenuation_factor_bottom * top_bottom
                        + right_left);
                self.cells[current_cell + 5] = 0.5
                    * (bottom_top - attenuation_factor_right * left_right
                        + attenuation_factor_bottom * top_bottom
                        + right_left);
                self.cells[current_cell + 6] = 0.5
                    * (bottom_top + attenuation_factor_right * left_right
                        - attenuation_factor_bottom * top_bottom
                        + right_left);
                self.cells[current_cell + 7] = 0.5
                    * (bottom_top
                        + attenuation_factor_right * left_right
                        + attenuation_factor_bottom * top_bottom
                        - right_left);
            }
        }
        //LeftBottom
        for x in 1..ui_state.e_al {
            for y in
                (SIMULATION_HEIGHT + ui_state.e_al)..(SIMULATION_HEIGHT + 2 * ui_state.e_al - 1)
            {
                let attenuation_factor_left = Grid::attenuation_factor(
                    ui_state.e_al,
                    ui_state.e_al - x,
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let attenuation_factor_bottom = Grid::attenuation_factor(
                    ui_state.e_al,
                    y - (SIMULATION_HEIGHT + ui_state.e_al),
                    ui_state.epsilon,
                    b,
                    ui_state.at_type,
                    ui_state.power_order,
                );

                let current_cell = Grid::coords_to_index(x, y, 0, ui_state.e_al);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2, ui_state.e_al)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3, ui_state.e_al)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0, ui_state.e_al)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1, ui_state.e_al)];
                self.cells[current_cell + 4] = 0.5
                    * (-bottom_top
                        + left_right
                        + attenuation_factor_bottom * top_bottom
                        + attenuation_factor_left * right_left);
                self.cells[current_cell + 5] = 0.5
                    * (bottom_top - left_right
                        + attenuation_factor_bottom * top_bottom
                        + attenuation_factor_left * right_left);
                self.cells[current_cell + 6] = 0.5
                    * (bottom_top + left_right - attenuation_factor_bottom * top_bottom
                        + attenuation_factor_left * right_left);
                self.cells[current_cell + 7] = 0.5
                    * (bottom_top + left_right + attenuation_factor_bottom * top_bottom
                        - attenuation_factor_left * right_left);
            }
        }
    }

    fn attenuation_factor(
        e_al: u32,
        distance: u32,
        epsilon: f32,
        b: f32,
        at_type: AttenuationType,
        power_order: u32,
    ) -> f32 {
        match at_type {
            AttenuationType::OriginalOneWay => {
                1.0 - ((1. + epsilon) - f32::exp((distance * distance) as f32 / b))
            }
            AttenuationType::Linear => 1.0 - ((distance) as f32 / e_al as f32).powi(1),
            AttenuationType::Power => {
                1.0 - ((distance) as f32 / e_al as f32).powi(power_order as i32)
            }
            // doesn't work
            AttenuationType::Old => {
                if distance == 1 {
                    -0.17157287525
                } else {
                    0.
                }
            }
            AttenuationType::DoNothing => 0.0,
        }
    }

    /// Calculates 1D array index from x,y coordinates (and an offset `index`)
    pub fn coords_to_index(x: u32, y: u32, index: u32, e_al: u32) -> usize {
        (y * (SIMULATION_WIDTH + 2 * e_al) * NUM_INDEX + x * NUM_INDEX + index) as usize
    }

    /// Calculates x,y coordinates from 1D array index
    pub fn index_to_coords(i: u32, e_al: u32) -> (u32, u32) {
        let x = (i / 9) % (SIMULATION_WIDTH + 2 * e_al);
        let y = i / 9 / (SIMULATION_WIDTH + 2 * e_al);
        (x, y)
    }
}

pub fn calc_system(mut grid: ResMut<Grid>, ui_state: Res<UiState>) {
    grid.calc(ui_state.e_al);
}

pub fn apply_system(
    mut grid: ResMut<Grid>,
    sources: Query<&Source>,
    microphones: Query<&mut Microphone>,
    walls: Query<&Wall>,
    game_ticks: Res<GameTicks>,
    ui_state: Res<UiState>,
) {
    grid.apply_sources(game_ticks.ticks_since_start, &sources, ui_state.e_al);
    grid.apply_walls(&walls, ui_state.e_al);
    grid.apply_microphones(game_ticks.ticks_since_start, microphones, ui_state.e_al);
    grid.apply_boundaries(ui_state);
}

pub fn update_system(
    mut grid: ResMut<Grid>,
    mut game_ticks: ResMut<GameTicks>,
    ui_state: Res<UiState>,
) {
    grid.update(ui_state.e_al);
    grid.update_delta_t(ui_state);
    game_ticks.ticks_since_start += 1;
}
