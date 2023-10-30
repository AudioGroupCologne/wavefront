use std::f32::consts::PI;

use bevy::prelude::*;

use crate::components::{Source, SourceType, Wall};
use crate::constants::*;

#[derive(Debug, Resource)]
pub struct Grid {
    /// full grid: [cur_bottom, cur_left, cur_top, cur_right, next_bottom, next_left, next_top, next_right, pressure]
    pub cells: Vec<f32>,
    /// A list of boundary nodes
    pub boundaries: Boundary,
}

#[derive(Debug, Default)]
pub struct Boundary {
    //Leo Smallvec und so
    /// indecies of bottom boundary nodes
    bottom: Vec<usize>,
    /// indecies of left boundary nodes
    left: Vec<usize>,
    /// indecies of top boundary nodes
    top: Vec<usize>,
    /// indecies of right boundary nodes
    right: Vec<usize>,
}

impl Default for Grid {
    fn default() -> Self {
        let mut grid = Self {
            cells: vec![0.; (SIMULATION_WIDTH * SIMULATION_HEIGHT * NUM_INDEX) as usize],
            boundaries: Default::default(),
        };
        grid.init_boundaries();
        grid
    }
}

impl Grid {
    fn update(&mut self) {
        for i in 0..SIMULATION_WIDTH * SIMULATION_HEIGHT {
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
        for x in 1..SIMULATION_WIDTH - 1 {
            for y in 1..SIMULATION_HEIGHT - 1 {
                self.calc_cell(
                    Grid::array_pos(x, y, 0),
                    self.cells[Grid::array_pos(x, y + 1, 2)],
                    self.cells[Grid::array_pos(x - 1, y, 3)],
                    self.cells[Grid::array_pos(x, y - 1, 0)],
                    self.cells[Grid::array_pos(x + 1, y, 1)],
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

    fn apply_sources(&mut self, time: f32, sources: &Query<&Source>) {
        for source in sources.iter() {
            //? maybe needs to be optimized
            let calc = match source.r#type {
                SourceType::Sin => {
                    source.amplitude * (2. * PI * source.frequency * (time - source.phase)).sin()
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
            let (x, y) = Grid::array_pos_rev(wall.0 as u32);
            self.cells[wall.0 + 4] = WALL_FAC * self.cells[Grid::array_pos(x, y + 1, 2)];
            self.cells[wall.0 + 5] = WALL_FAC * self.cells[Grid::array_pos(x - 1, y, 3)];
            self.cells[wall.0 + 6] = WALL_FAC * self.cells[Grid::array_pos(x, y - 1, 0)];
            self.cells[wall.0 + 7] = WALL_FAC * self.cells[Grid::array_pos(x + 1, y, 1)];
        }
    }

    fn apply_boundaries(&mut self) {
        //pls math check
        for &boundary_index in self.boundaries.bottom.iter() {
            self.cells[boundary_index + 6] =
                BOUNDARY_FAC * self.cells[boundary_index - (NUM_INDEX * SIMULATION_WIDTH) as usize];
        }
        for &boundary_index in self.boundaries.left.iter() {
            self.cells[boundary_index + 7] =
                BOUNDARY_FAC * self.cells[boundary_index + NUM_INDEX as usize + 1];
        }
        for &boundary_index in self.boundaries.top.iter() {
            self.cells[boundary_index + 4] = BOUNDARY_FAC
                * self.cells[boundary_index + (NUM_INDEX * SIMULATION_WIDTH) as usize + 2];
        }
        for &boundary_index in self.boundaries.right.iter() {
            self.cells[boundary_index + 5] =
                BOUNDARY_FAC * self.cells[boundary_index - NUM_INDEX as usize + 3];
        }
    }

    pub fn init_boundaries(&mut self) {
        // TOP
        for x in 0..SIMULATION_WIDTH {
            self.boundaries.top.push(Grid::array_pos(x, 0, 0))
        }
        // BOTTOM
        for x in 0..SIMULATION_WIDTH {
            self.boundaries
                .bottom
                .push(Grid::array_pos(x, SIMULATION_HEIGHT - 1, 0))
        }
        // LEFT
        for y in 0..SIMULATION_HEIGHT {
            self.boundaries.left.push(Grid::array_pos(0, y, 0))
        }
        // RIGHT
        for y in 0..SIMULATION_HEIGHT {
            self.boundaries
                .right
                .push(Grid::array_pos(SIMULATION_WIDTH - 1, y, 0))
        }
    }

    //TODO: doc string and maybe rename (coords_to_index)?
    pub fn array_pos(x: u32, y: u32, index: u32) -> usize {
        (y * SIMULATION_WIDTH * NUM_INDEX + x * NUM_INDEX + index) as usize
    }

    //TODO: doc string and maybe rename (index_to_coords)?
    pub fn array_pos_rev(i: u32) -> (u32, u32) {
        let x = (i / 9) % SIMULATION_WIDTH;
        let y = i / 9 / SIMULATION_WIDTH;
        (x, y)
    }
}

pub fn calc_system(mut grid: ResMut<Grid>) {
    grid.calc();
}

pub fn apply_system(
    mut grid: ResMut<Grid>,
    time: Res<Time>,
    sources: Query<&Source>,
    walls: Query<&Wall>,
) {
    grid.apply_sources(time.elapsed_seconds(), &sources);
    grid.apply_walls(&walls);
    grid.apply_boundaries();
}

pub fn update_system(mut grid: ResMut<Grid>) {
    grid.update();
}
