use std::f32::consts::TAU;

use bevy::prelude::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use super::plugin::WaveSamples;
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::{CircWall, RectWall, Wall, WallCell};
use crate::math::constants::*;
use crate::math::transformations::{coords_to_index, index_to_coords};
use crate::ui::state::UiState;

#[derive(Clone, Copy, Debug, Default)]
pub struct Cell {
    pub bottom: f32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
}

#[derive(Debug, Resource)]
pub struct Grid {
    /// Grid cells
    pub cur_cells: Vec<Cell>,
    pub next_cells: Vec<Cell>,
    pub pressure: Vec<f32>,
    pub wall_cache: Vec<WallCell>,
    boundary_cache: Vec<[f32; 4]>,
    /// Delta t in seconds
    pub delta_t: f32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cur_cells: vec![
                Cell::default();
                ((SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH)
                    * (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH))
                    as usize
            ],
            next_cells: vec![
                Cell::default();
                ((SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH)
                    * (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH))
                    as usize
            ],
            pressure: vec![
                0_f32;
                ((SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH)
                    * (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH))
                    as usize
            ],
            wall_cache: vec![
                WallCell::default();
                ((SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH)
                    * (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH))
                    as usize
            ],
            boundary_cache: vec![
                [0_f32; 4];
                ((SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH)
                    * (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH))
                    as usize
            ],
            // set to result in a sample rate of 48kHz
            delta_t: 0.00715 / PROPAGATION_SPEED,
        }
    }
}

impl Grid {
    pub fn update_delta_t(&mut self, delta_l: f32) {
        self.delta_t = delta_l / PROPAGATION_SPEED;
    }

    pub fn reset_cells(&mut self, boundary_width: u32) {
        self.cur_cells = vec![
            Cell::default();
            ((SIMULATION_WIDTH + 2 * boundary_width) * (SIMULATION_HEIGHT + 2 * boundary_width))
                as usize
        ];
        self.next_cells = vec![
            Cell::default();
            ((SIMULATION_WIDTH + 2 * boundary_width) * (SIMULATION_HEIGHT + 2 * boundary_width))
                as usize
        ];
        self.pressure = vec![
            0_f32;
            ((SIMULATION_WIDTH + 2 * boundary_width) * (SIMULATION_HEIGHT + 2 * boundary_width))
                as usize
        ];
    }

    // this needs to be called when changing the boundary_width
    pub fn reset_walls(&mut self, boundary_width: u32) {
        self.wall_cache = vec![
            WallCell::default();
            ((SIMULATION_WIDTH + 2 * boundary_width) * (SIMULATION_HEIGHT + 2 * boundary_width))
                as usize
        ];
    }

    pub fn update_cells(&mut self) {
        self.cur_cells
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, cur_cell)| {
                cur_cell.bottom = self.next_cells[index].bottom;
                cur_cell.left = self.next_cells[index].left;
                cur_cell.top = self.next_cells[index].top;
                cur_cell.right = self.next_cells[index].right;
            });

        self.pressure
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, pressure)| {
                *pressure = 0.5
                    * (self.cur_cells[index].bottom
                        + self.cur_cells[index].left
                        + self.cur_cells[index].top
                        + self.cur_cells[index].right);
            });
    }

    pub fn update_walls(
        &mut self,
        rect_walls: &Query<&RectWall>,
        circ_walls: &Query<&CircWall>,
        boundary_width: u32,
    ) {
        self.wall_cache.par_iter_mut().for_each(|wall_cell| {
            wall_cell.is_wall = false;
        });

        self.wall_cache
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, wall_cell)| {
                let (x, y) = index_to_coords(index as u32, boundary_width);

                for wall in rect_walls {
                    if wall.edge_contains(
                        x.saturating_sub(boundary_width),
                        y.saturating_sub(boundary_width),
                    ) {
                        wall_cell.is_wall = true;
                        wall_cell.reflection_factor = wall.get_reflection_factor();
                        wall_cell.draw_reflection_factor = wall.get_reflection_factor();
                    } else if wall.contains(
                        x.saturating_sub(boundary_width),
                        y.saturating_sub(boundary_width),
                    ) || wall.boundary_delete(x, y, boundary_width)
                    {
                        wall_cell.is_wall = true;
                        wall_cell.reflection_factor = 0.;
                        wall_cell.draw_reflection_factor = wall.get_reflection_factor();
                    }
                }

                for wall in circ_walls {
                    if wall.contains(
                        x.saturating_sub(boundary_width),
                        y.saturating_sub(boundary_width),
                    ) || wall.boundary_delete(x, y, boundary_width)
                    {
                        wall_cell.is_wall = true;
                        wall_cell.reflection_factor = 0.;
                        wall_cell.draw_reflection_factor = wall.get_reflection_factor();
                    }
                }
            });

        for wall in circ_walls {
            let mut b_x = 0i32;
            let mut b_y = wall.radius as i32;
            let mut d = 1 - wall.radius as i32;
            while b_x <= b_y {
                for (x, y) in [
                    (wall.center.x as i32 + b_x, wall.center.y as i32 + b_y),
                    (wall.center.x as i32 + b_x, wall.center.y as i32 - b_y),
                    (wall.center.x as i32 - b_x, wall.center.y as i32 + b_y),
                    (wall.center.x as i32 - b_x, wall.center.y as i32 - b_y),
                    (wall.center.x as i32 + b_y, wall.center.y as i32 + b_x),
                    (wall.center.x as i32 + b_y, wall.center.y as i32 - b_x),
                    (wall.center.x as i32 - b_y, wall.center.y as i32 + b_x),
                    (wall.center.x as i32 - b_y, wall.center.y as i32 - b_x),
                ] {
                    let x = (x + boundary_width as i32) as u32;
                    let y = (y + boundary_width as i32) as u32;
                    if x < SIMULATION_WIDTH + 2 * boundary_width
                        && y < SIMULATION_HEIGHT + 2 * boundary_width
                    {
                        // angle in [0, 2pi)
                        let mut angle =
                            if (y as i32 - wall.center.y as i32 - boundary_width as i32) <= 0 {
                                ((x as f32 - wall.center.x as f32 - boundary_width as f32)
                                    / wall.radius as f32)
                                    .acos()
                            } else {
                                TAU - ((x as f32 - wall.center.x as f32 - boundary_width as f32)
                                    / wall.radius as f32)
                                    .acos()
                            };

                        angle = (angle + wall.rotation_angle.to_radians()) % TAU;

                        if angle >= wall.open_circ_segment.to_radians() / 2.
                            && angle <= TAU - wall.open_circ_segment.to_radians() / 2.
                            || !wall.is_hollow
                        {
                            let index = coords_to_index(x, y, boundary_width);
                            self.wall_cache[index].is_wall = true;
                            self.wall_cache[index].reflection_factor = wall.get_reflection_factor();
                            self.wall_cache[index].draw_reflection_factor =
                                wall.get_reflection_factor();
                        }
                    }
                }

                if d < 0 {
                    d = d + 2 * b_x + 3;
                    b_x += 1;
                } else {
                    d = d + 2 * (b_x - b_y) + 5;
                    b_x += 1;
                    b_y -= 1;
                }
            }
        }
    }

    /// Update all cells in the grid by calculating cell reflection pulses
    pub fn calc_cells(&mut self, boundary_width: u32) {
        self.next_cells
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, next_cell)| {
                let (x, y) = index_to_coords(index as u32, boundary_width);
                if x > 0
                    && x < SIMULATION_WIDTH + 2 * boundary_width - 1
                    && y > 0
                    && y < SIMULATION_HEIGHT + 2 * boundary_width - 1
                {
                    let bottom_cell = self.cur_cells[coords_to_index(x, y + 1, boundary_width)];
                    let left_cell = self.cur_cells[coords_to_index(x - 1, y, boundary_width)];
                    let top_cell = self.cur_cells[coords_to_index(x, y - 1, boundary_width)];
                    let right_cell = self.cur_cells[coords_to_index(x + 1, y, boundary_width)];
                    if self.wall_cache[index].is_wall {
                        let reflection_factor = self.wall_cache[index].reflection_factor;
                        next_cell.bottom = reflection_factor * bottom_cell.top;
                        next_cell.left = reflection_factor * left_cell.right;
                        next_cell.top = reflection_factor * top_cell.bottom;
                        next_cell.right = reflection_factor * right_cell.left;
                    } else if x >= boundary_width
                        && x < SIMULATION_WIDTH + boundary_width
                        && y < SIMULATION_HEIGHT + boundary_width
                        && y >= boundary_width
                    {
                        // if pixel is in sim region
                        next_cell.bottom = 0.5
                            * (-bottom_cell.top
                                + left_cell.right
                                + top_cell.bottom
                                + right_cell.left);
                        next_cell.left = 0.5
                            * (bottom_cell.top - left_cell.right
                                + top_cell.bottom
                                + right_cell.left);
                        next_cell.top = 0.5
                            * (bottom_cell.top + left_cell.right - top_cell.bottom
                                + right_cell.left);
                        next_cell.right = 0.5
                            * (bottom_cell.top + left_cell.right + top_cell.bottom
                                - right_cell.left);
                    } else {
                        // this pixels is in the boundary

                        let factors = self.boundary_cache[index];

                        //TODO: Maybe better encoding for at_factors (lookup table, index pos?)

                        next_cell.bottom = 0.5
                            * (-factors[0] * bottom_cell.top
                                + factors[1] * left_cell.right
                                + factors[2] * top_cell.bottom
                                + factors[3] * right_cell.left);
                        next_cell.left = 0.5
                            * (factors[0] * bottom_cell.top - factors[1] * left_cell.right
                                + factors[2] * top_cell.bottom
                                + factors[3] * right_cell.left);
                        next_cell.top = 0.5
                            * (factors[0] * bottom_cell.top + factors[1] * left_cell.right
                                - factors[2] * top_cell.bottom
                                + factors[3] * right_cell.left);
                        next_cell.right = 0.5
                            * (factors[0] * bottom_cell.top
                                + factors[1] * left_cell.right
                                + factors[2] * top_cell.bottom
                                - factors[3] * right_cell.left);
                    }
                }
            });
    }

    /// Write source outputs into cell reflection pulses
    pub fn apply_sources(
        &mut self,
        time_since_start: f32,
        samples_since_start: usize,
        sources: &Query<&Source>,
        boundary_width: u32,
        wave_samples: &WaveSamples,
    ) {
        for source in sources.iter() {
            let calc = source.calc(time_since_start, samples_since_start, wave_samples);
            let source_pos = coords_to_index(
                source.x + boundary_width,
                source.y + boundary_width,
                boundary_width,
            );
            self.next_cells[source_pos].bottom += calc;
            self.next_cells[source_pos].left += calc;
            self.next_cells[source_pos].top += calc;
            self.next_cells[source_pos].right += calc;
        }
    }

    /// If plots are enabled, write cell pressure values into microphones
    pub fn apply_microphones(
        &self,
        mut microphones: Query<&mut Microphone>,
        ui_state: &UiState,
        time_since_start: f64,
    ) {
        if ui_state.show_plots {
            for mut mic in microphones.iter_mut() {
                let x = mic.x;
                let y = mic.y;

                mic.record.push([
                    time_since_start,
                    self.pressure[coords_to_index(
                        x + ui_state.boundary_width,
                        y + ui_state.boundary_width,
                        ui_state.boundary_width,
                    )] as f64,
                ]);
            }
        }
    }

    pub fn cache_boundaries(&mut self, boundary_width: u32) {
        self.boundary_cache = vec![
            [0_f32; 4];
            ((SIMULATION_WIDTH + 2 * boundary_width) * (SIMULATION_HEIGHT + 2 * boundary_width))
                as usize
        ];
        // going in 'rings' from outer to inner
        // every ring shares an attenuation factor
        for r in 1..boundary_width {
            // there was a '?' in front of ui_state, that's not needed right?
            // also distance could be just r -> need to redo att_fac calcs
            let attenuation_factor =
                Grid::attenuation_factor(boundary_width, 5, boundary_width - r);

            // bottom
            for x in r..(SIMULATION_WIDTH + 2 * boundary_width - r) {
                let y = SIMULATION_HEIGHT + 2 * boundary_width - r - 1;
                let current_cell_index = coords_to_index(x, y, boundary_width);

                self.boundary_cache[current_cell_index] = [1., 1., attenuation_factor, 1.];

                // [1., 1., at, 1.]
            }
            // left
            for y in r..(SIMULATION_HEIGHT + 2 * boundary_width - r) {
                let x = r;
                let current_cell_index = coords_to_index(x, y, boundary_width);

                self.boundary_cache[current_cell_index] = [1., 1., 1., attenuation_factor];

                // [1., 1., 1., at]
            }
            // top
            for x in r..(SIMULATION_WIDTH + 2 * boundary_width - r) {
                let y = r;
                let current_cell_index = coords_to_index(x, y, boundary_width);

                self.boundary_cache[current_cell_index] = [attenuation_factor, 1., 1., 1.];

                // [at, 1., 1., 1.]
            }
            // right
            for y in r..(SIMULATION_HEIGHT + 2 * boundary_width - r) {
                let x = SIMULATION_WIDTH + 2 * boundary_width - r - 1;
                let current_cell_index = coords_to_index(x, y, boundary_width);

                self.boundary_cache[current_cell_index] = [1., attenuation_factor, 1., 1.];

                // [1., at, 1., 1.]
            }
        }
    }

    /// Get boundary attenuation factor based on distance
    fn attenuation_factor(boundary_width: u32, power_order: u32, distance: u32) -> f32 {
        1.0 - (distance as f32 / boundary_width as f32).powi(power_order as i32)
    }
}
