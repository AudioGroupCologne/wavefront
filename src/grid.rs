use std::f32::consts::PI;

use bevy::prelude::*;
use smallvec::SmallVec;

use crate::components::{GameTicks, Source, SourceType, Wall};
use crate::constants::*;
use crate::render::UiState;

#[derive(Debug, Resource)]
pub struct Grid {
    /// full grid: [cur_bottom, cur_left, cur_top, cur_right, next_bottom, next_left, next_top, next_right, pressure]
    pub cells: Vec<f32>,
    /// A list of boundary nodes
    pub boundaries: Boundary,
    /// Delta L in meters
    pub delta_l: f32,
    /// Delta s in seconds
    pub delta_t: f32,
}

#[derive(Debug, Default)]
pub struct Boundary {
    /// indecies of bottom boundary nodes
    bottom: SmallVec<[usize; SIMULATION_WIDTH as usize]>,
    /// indecies of left boundary nodes
    left: SmallVec<[usize; SIMULATION_HEIGHT as usize]>,
    /// indecies of top boundary nodes
    top: SmallVec<[usize; SIMULATION_WIDTH as usize]>,
    /// indecies of right boundary nodes
    right: SmallVec<[usize; SIMULATION_HEIGHT as usize]>,
}

impl Default for Grid {
    fn default() -> Self {
        let mut grid = Self {
            cells: vec![
                0.;
                ((SIMULATION_WIDTH + 2 * E_AL) * (SIMULATION_HEIGHT + 2 * E_AL) * NUM_INDEX)
                    as usize
            ],
            boundaries: Default::default(),
            delta_l: 0.001, //basically useless when using val from ui
            delta_t: 0.001 / PROPAGATION_SPEED,
        };
        grid.init_boundaries();
        grid
    }
}

impl Grid {
    fn update_delta_t(&mut self, ui_state: Res<UiState>) {
        self.delta_t = ui_state.delta_l / PROPAGATION_SPEED;
    }

    fn update(&mut self) {
        for i in 0..(SIMULATION_WIDTH + 2 * E_AL) * (SIMULATION_HEIGHT + 2 * E_AL) {
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

    fn calc(&mut self) {
        for x in E_AL..(SIMULATION_WIDTH + E_AL) {
            for y in E_AL..(SIMULATION_HEIGHT + E_AL) {
                self.calc_cell(
                    Grid::coords_to_index(x, y, 0),
                    self.cells[Grid::coords_to_index(x, y + 1, 2)],
                    self.cells[Grid::coords_to_index(x - 1, y, 3)],
                    self.cells[Grid::coords_to_index(x, y - 1, 0)],
                    self.cells[Grid::coords_to_index(x + 1, y, 1)],
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

    fn apply_sources(&mut self, ticks_since_start: u64, sources: &Query<&Source>) {
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

            self.cells[source.index + 4] = calc;
            self.cells[source.index + 5] = calc;
            self.cells[source.index + 6] = calc;
            self.cells[source.index + 7] = calc;
        }
    }

    fn apply_walls(&mut self, walls: &Query<&Wall>) {
        for wall in walls.iter() {
            let (x, y) = Grid::index_to_coords(wall.0 as u32);
            self.cells[wall.0 + 4] = WALL_FAC * self.cells[Grid::coords_to_index(x, y + 1, 2)];
            self.cells[wall.0 + 5] = WALL_FAC * self.cells[Grid::coords_to_index(x - 1, y, 3)];
            self.cells[wall.0 + 6] = WALL_FAC * self.cells[Grid::coords_to_index(x, y - 1, 0)];
            self.cells[wall.0 + 7] = WALL_FAC * self.cells[Grid::coords_to_index(x + 1, y, 1)];
        }
    }

    fn apply_boundaries(&mut self, ui_state: Res<UiState>) {
        let b = (E_AL * E_AL) as f32 / (f32::ln(ui_state.epsilon));
        //Left
        for x in 1..E_AL {
            for y in E_AL..(SIMULATION_HEIGHT + E_AL) {
                // let attenuation_factor = 1.0
                //     - ((1. + ui_state.epsilon) - f32::exp(((E_AL - x) * (E_AL - x)) as f32 / b));
                let attenuation_factor = 1.0 - ((E_AL - x) as f32 / E_AL as f32).powi(5); //different attenuation_factor, impl switch or sth for testing?
                let current_cell = Grid::coords_to_index(x, y, 0);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1)];
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
        for x in E_AL..(SIMULATION_WIDTH + E_AL) {
            for y in 1..E_AL {
                let attenuation_factor = 1.0
                    - ((1. + ui_state.epsilon) - f32::exp(((E_AL - y) * (E_AL - y)) as f32 / b));
                let current_cell = Grid::coords_to_index(x, y, 0);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1)];
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
        for x in (SIMULATION_WIDTH + E_AL)..(SIMULATION_WIDTH + 2 * E_AL - 1) {
            for y in E_AL..(SIMULATION_HEIGHT + E_AL) {
                let attenuation_factor = 1.0
                    - ((1. + ui_state.epsilon)
                        - f32::exp(
                            ((x - (SIMULATION_WIDTH + E_AL)) * (x - (SIMULATION_WIDTH + E_AL)))
                                as f32
                                / b,
                        ));
                let current_cell = Grid::coords_to_index(x, y, 0);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1)];
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
        for x in E_AL..(SIMULATION_WIDTH + E_AL) {
            for y in (SIMULATION_HEIGHT + E_AL)..(SIMULATION_HEIGHT + 2 * E_AL - 1) {
                let attenuation_factor = 1.0
                    - ((1. + ui_state.epsilon)
                        - f32::exp(
                            ((y - (SIMULATION_HEIGHT + E_AL)) * (y - (SIMULATION_HEIGHT + E_AL)))
                                as f32
                                / b,
                        ));
                let current_cell = Grid::coords_to_index(x, y, 0);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1)];
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
        for x in 1..E_AL {
            for y in 1..E_AL {
                let attenuation_factor_left = 1.0
                    - ((1. + ui_state.epsilon) - f32::exp(((E_AL - x) * (E_AL - x)) as f32 / b));
                let attenuation_factor_top = 1.0
                    - ((1. + ui_state.epsilon) - f32::exp(((E_AL - y) * (E_AL - y)) as f32 / b));
                let current_cell = Grid::coords_to_index(x, y, 0);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1)];
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
        for x in (SIMULATION_WIDTH + E_AL)..(SIMULATION_WIDTH + 2 * E_AL - 1) {
            for y in 1..E_AL {
                let attenuation_factor_right = 1.0
                    - ((1. + ui_state.epsilon)
                        - f32::exp(
                            ((x - (SIMULATION_WIDTH + E_AL)) * (x - (SIMULATION_WIDTH + E_AL)))
                                as f32
                                / b,
                        ));
                let attenuation_factor_top = 1.0
                    - ((1. + ui_state.epsilon) - f32::exp(((E_AL - y) * (E_AL - y)) as f32 / b));
                let current_cell = Grid::coords_to_index(x, y, 0);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1)];
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
        for x in (SIMULATION_WIDTH + E_AL)..(SIMULATION_WIDTH + 2 * E_AL - 1) {
            for y in (SIMULATION_HEIGHT + E_AL)..(SIMULATION_HEIGHT + 2 * E_AL - 1) {
                let attenuation_factor_right = 1.0
                    - ((1. + ui_state.epsilon)
                        - f32::exp(
                            ((x - (SIMULATION_WIDTH + E_AL)) * (x - (SIMULATION_WIDTH + E_AL)))
                                as f32
                                / b,
                        ));
                let attenuation_factor_bottom = 1.0
                    - ((1. + ui_state.epsilon)
                        - f32::exp(
                            ((y - (SIMULATION_HEIGHT + E_AL)) * (y - (SIMULATION_HEIGHT + E_AL)))
                                as f32
                                / b,
                        ));
                let current_cell = Grid::coords_to_index(x, y, 0);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1)];
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
        for x in 1..E_AL {
            for y in (SIMULATION_HEIGHT + E_AL)..(SIMULATION_HEIGHT + 2 * E_AL - 1) {
                let attenuation_factor_left = 1.0
                    - ((1. + ui_state.epsilon) - f32::exp(((E_AL - x) * (E_AL - x)) as f32 / b));
                let attenuation_factor_bottom = 1.0
                    - ((1. + ui_state.epsilon)
                        - f32::exp(
                            ((y - (SIMULATION_HEIGHT + E_AL)) * (y - (SIMULATION_HEIGHT + E_AL)))
                                as f32
                                / b,
                        ));
                let current_cell = Grid::coords_to_index(x, y, 0);
                let bottom_top = self.cells[Grid::coords_to_index(x, y + 1, 2)];
                let left_right = self.cells[Grid::coords_to_index(x - 1, y, 3)];
                let top_bottom = self.cells[Grid::coords_to_index(x, y - 1, 0)];
                let right_left = self.cells[Grid::coords_to_index(x + 1, y, 1)];
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

    pub fn init_boundaries(&mut self) {
        // TOP
        for x in 0..SIMULATION_WIDTH {
            self.boundaries.top.push(Grid::coords_to_index(x, 0, 0))
        }
        // BOTTOM
        for x in 0..SIMULATION_WIDTH {
            self.boundaries
                .bottom
                .push(Grid::coords_to_index(x, SIMULATION_HEIGHT - 1, 0))
        }
        // LEFT
        for y in 0..SIMULATION_HEIGHT {
            self.boundaries.left.push(Grid::coords_to_index(0, y, 0))
        }
        // RIGHT
        for y in 0..SIMULATION_HEIGHT {
            self.boundaries
                .right
                .push(Grid::coords_to_index(SIMULATION_WIDTH - 1, y, 0))
        }
    }

    /// Calculates 1D array index from x,y coordinates (and an offset `index`)
    pub fn coords_to_index(x: u32, y: u32, index: u32) -> usize {
        (y * (SIMULATION_WIDTH + 2 * E_AL) * NUM_INDEX + x * NUM_INDEX + index) as usize
    }

    /// Calculates x,y coordinates from 1D array index
    pub fn index_to_coords(i: u32) -> (u32, u32) {
        let x = (i / 9) % (SIMULATION_WIDTH + 2 * E_AL);
        let y = i / 9 / (SIMULATION_WIDTH + 2 * E_AL);
        (x, y)
    }
}

pub fn calc_system(mut grid: ResMut<Grid>) {
    grid.calc();
}

pub fn apply_system(
    mut grid: ResMut<Grid>,
    sources: Query<&Source>,
    walls: Query<&Wall>,
    game_ticks: Res<GameTicks>,
    ui_state: Res<UiState>,
) {
    grid.apply_sources(game_ticks.ticks_since_start, &sources);
    grid.apply_walls(&walls);
    grid.apply_boundaries(ui_state);
}

pub fn update_system(
    mut grid: ResMut<Grid>,
    mut game_ticks: ResMut<GameTicks>,
    ui_state: Res<UiState>,
) {
    grid.update();
    grid.update_delta_t(ui_state);
    game_ticks.ticks_since_start += 1;
}
