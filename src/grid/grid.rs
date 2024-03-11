use bevy::prelude::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::states::Overlay;
use crate::components::wall::{CircWall, RectWall, Wall};
use crate::math::constants::*;
use crate::math::transformations::{coords_to_index, index_to_coords};
use crate::ui::state::{AttenuationType, UiState};

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub bottom: f32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            bottom: 0.,
            left: 0.,
            top: 0.,
            right: 0.,
        }
    }
}

#[derive(Debug, Resource)]
pub struct Grid {
    /// Grid cells
    pub cur_cells: Vec<Cell>,
    pub next_cells: Vec<Cell>,
    pub pressure: Vec<f32>,
    /// Delta s in seconds
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
            delta_t: 0.001 / PROPAGATION_SPEED,
        }
    }
}

impl Grid {
    pub fn update_delta_t(&mut self, ui_state: &UiState) {
        self.delta_t = ui_state.delta_l / PROPAGATION_SPEED;
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

    pub fn calc_cells(
        &mut self,
        rect_walls: &Query<&RectWall, Without<Overlay>>,
        circ_walls: &Query<&CircWall, Without<Overlay>>,
        boundary_width: u32,
    ) {
        self.next_cells
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, next_cell)| {
                let (x, y) = index_to_coords(index as u32, boundary_width);
                // if pixel is in sim region
                if x >= boundary_width
                    && x < SIMULATION_WIDTH + boundary_width
                    && y >= boundary_width
                    && y < SIMULATION_HEIGHT + boundary_width
                {
                    let bottom_cell = self.cur_cells[coords_to_index(x, y + 1, boundary_width)];
                    let left_cell = self.cur_cells[coords_to_index(x - 1, y, boundary_width)];
                    let top_cell = self.cur_cells[coords_to_index(x, y - 1, boundary_width)];
                    let right_cell = self.cur_cells[coords_to_index(x + 1, y, boundary_width)];

                    // theoretically more calculations than needed, needs more thinking
                    next_cell.bottom = 0.5
                        * (-bottom_cell.top + left_cell.right + top_cell.bottom + right_cell.left);
                    next_cell.left = 0.5
                        * (bottom_cell.top - left_cell.right + top_cell.bottom + right_cell.left);
                    next_cell.top = 0.5
                        * (bottom_cell.top + left_cell.right - top_cell.bottom + right_cell.left);
                    next_cell.right = 0.5
                        * (bottom_cell.top + left_cell.right + top_cell.bottom - right_cell.left);

                    for wall in rect_walls.iter() {
                        if wall.contains(x, y) {
                            next_cell.bottom = 0.;
                            next_cell.left = 0.;
                            next_cell.top = 0.;
                            next_cell.right = 0.;
                        }
                        if wall.edge_contains(x, y) {
                            next_cell.bottom = wall.reflection_factor
                                * self.cur_cells[coords_to_index(x, y + 1, boundary_width)].top;
                            next_cell.left = wall.reflection_factor
                                * self.cur_cells[coords_to_index(x - 1, y, boundary_width)].right;
                            next_cell.top = wall.reflection_factor
                                * self.cur_cells[coords_to_index(x, y - 1, boundary_width)].bottom;
                            next_cell.right = wall.reflection_factor
                                * self.cur_cells[coords_to_index(x + 1, y, boundary_width)].left;
                        }
                    }
                }
            });
    }

    pub fn apply_sources(
        &mut self,
        ticks_since_start: u64,
        sources: &Query<&Source>,
        boundary_width: u32,
    ) {
        let time = self.delta_t * ticks_since_start as f32; //the cast feels wrong, but it works for now
        for source in sources.iter() {
            let calc = source.calc(time);
            let source_pos = coords_to_index(
                source.x + boundary_width,
                source.y + boundary_width,
                boundary_width,
            );
            self.next_cells[source_pos].bottom = calc;
            self.next_cells[source_pos].left = calc;
            self.next_cells[source_pos].top = calc;
            self.next_cells[source_pos].right = calc;
        }
    }

    pub fn apply_microphones(&self, mut microphones: Query<&mut Microphone>, ui_state: &UiState) {
        if ui_state.show_plots {
            for mut mic in microphones.iter_mut() {
                let x = mic.x;
                let y = mic.y;
                let cur_time = mic.record.last().unwrap()[0] + self.delta_t as f64;

                mic.record.push([
                    cur_time,
                    self.pressure[coords_to_index(x, y, ui_state.boundary_width)] as f64,
                ]);
            }
        }
    }

    pub fn apply_boundaries(&mut self, ui_state: &UiState) {
        let b = (ui_state.boundary_width * ui_state.boundary_width) as f32 / ui_state.epsilon.ln();
        // going in 'rings' from outer to inner
        // every ring shares an attenuation factor
        for r in 1..ui_state.boundary_width {
            // there was a '?' in front of ui_state, that's not needed right?
            // also distance could be just r -> need to redo att_fac calcs
            let attenuation_factor =
                Grid::attenuation_factor(ui_state, ui_state.boundary_width - r, b);

            // bottom
            for x in r..(SIMULATION_WIDTH + 2 * ui_state.boundary_width - r) {
                let y = SIMULATION_HEIGHT + 2 * ui_state.boundary_width - r - 1;
                let current_cell_index = coords_to_index(x, y, ui_state.boundary_width);
                let bottom_cell =
                    self.cur_cells[coords_to_index(x, y + 1, ui_state.boundary_width)];
                let left_cell = self.cur_cells[coords_to_index(x - 1, y, ui_state.boundary_width)];
                let top_cell = self.cur_cells[coords_to_index(x, y - 1, ui_state.boundary_width)];
                let right_cell = self.cur_cells[coords_to_index(x + 1, y, ui_state.boundary_width)];

                self.next_cells[current_cell_index].bottom = 0.5
                    * (-bottom_cell.top
                        + left_cell.right
                        + attenuation_factor * top_cell.bottom
                        + right_cell.left);
                self.next_cells[current_cell_index].left = 0.5
                    * (bottom_cell.top - left_cell.right
                        + attenuation_factor * top_cell.bottom
                        + right_cell.left);
                self.next_cells[current_cell_index].top = 0.5
                    * (bottom_cell.top + left_cell.right - attenuation_factor * top_cell.bottom
                        + right_cell.left);
                self.next_cells[current_cell_index].right = 0.5
                    * (bottom_cell.top + left_cell.right + attenuation_factor * top_cell.bottom
                        - right_cell.left);
            }
            // left
            for y in r..(SIMULATION_HEIGHT + 2 * ui_state.boundary_width - r) {
                let x = r;
                let current_cell_index = coords_to_index(x, y, ui_state.boundary_width);
                let bottom_cell =
                    self.cur_cells[coords_to_index(x, y + 1, ui_state.boundary_width)];
                let left_cell = self.cur_cells[coords_to_index(x - 1, y, ui_state.boundary_width)];
                let top_cell = self.cur_cells[coords_to_index(x, y - 1, ui_state.boundary_width)];
                let right_cell = self.cur_cells[coords_to_index(x + 1, y, ui_state.boundary_width)];

                self.next_cells[current_cell_index].bottom = 0.5
                    * (-bottom_cell.top
                        + left_cell.right
                        + top_cell.bottom
                        + attenuation_factor * right_cell.left);
                self.next_cells[current_cell_index].left = 0.5
                    * (bottom_cell.top - left_cell.right
                        + top_cell.bottom
                        + attenuation_factor * right_cell.left);
                self.next_cells[current_cell_index].top = 0.5
                    * (bottom_cell.top + left_cell.right - top_cell.bottom
                        + attenuation_factor * right_cell.left);
                self.next_cells[current_cell_index].right = 0.5
                    * (bottom_cell.top + left_cell.right + top_cell.bottom
                        - attenuation_factor * right_cell.left);
            }
            // top
            for x in r..(SIMULATION_WIDTH + 2 * ui_state.boundary_width - r) {
                let y = r;
                let current_cell_index = coords_to_index(x, y, ui_state.boundary_width);
                let bottom_cell =
                    self.cur_cells[coords_to_index(x, y + 1, ui_state.boundary_width)];
                let left_cell = self.cur_cells[coords_to_index(x - 1, y, ui_state.boundary_width)];
                let top_cell = self.cur_cells[coords_to_index(x, y - 1, ui_state.boundary_width)];
                let right_cell = self.cur_cells[coords_to_index(x + 1, y, ui_state.boundary_width)];

                self.next_cells[current_cell_index].bottom = 0.5
                    * (attenuation_factor * -bottom_cell.top
                        + left_cell.right
                        + top_cell.bottom
                        + right_cell.left);
                self.next_cells[current_cell_index].left = 0.5
                    * (attenuation_factor * bottom_cell.top - left_cell.right
                        + top_cell.bottom
                        + right_cell.left);
                self.next_cells[current_cell_index].top = 0.5
                    * (attenuation_factor * bottom_cell.top + left_cell.right - top_cell.bottom
                        + right_cell.left);
                self.next_cells[current_cell_index].right = 0.5
                    * (attenuation_factor * bottom_cell.top + left_cell.right + top_cell.bottom
                        - right_cell.left);
            }
            // right
            for y in r..(SIMULATION_HEIGHT + 2 * ui_state.boundary_width - r) {
                let x = SIMULATION_WIDTH + 2 * ui_state.boundary_width - r - 1;
                let current_cell_index = coords_to_index(x, y, ui_state.boundary_width);
                let bottom_cell =
                    self.cur_cells[coords_to_index(x, y + 1, ui_state.boundary_width)];
                let left_cell = self.cur_cells[coords_to_index(x - 1, y, ui_state.boundary_width)];
                let top_cell = self.cur_cells[coords_to_index(x, y - 1, ui_state.boundary_width)];
                let right_cell = self.cur_cells[coords_to_index(x + 1, y, ui_state.boundary_width)];

                self.next_cells[current_cell_index].bottom = 0.5
                    * (-bottom_cell.top
                        + attenuation_factor * left_cell.right
                        + top_cell.bottom
                        + right_cell.left);
                self.next_cells[current_cell_index].left = 0.5
                    * (bottom_cell.top - attenuation_factor * left_cell.right
                        + top_cell.bottom
                        + right_cell.left);
                self.next_cells[current_cell_index].top = 0.5
                    * (bottom_cell.top + attenuation_factor * left_cell.right - top_cell.bottom
                        + right_cell.left);
                self.next_cells[current_cell_index].right = 0.5
                    * (bottom_cell.top + attenuation_factor * left_cell.right + top_cell.bottom
                        - right_cell.left);
            }
        }
    }

    fn attenuation_factor(ui_state: &UiState, distance: u32, b: f32) -> f32 {
        match ui_state.at_type {
            AttenuationType::OriginalOneWay => {
                1.0 - ((1. + ui_state.epsilon) - ((distance * distance) as f32 / b).exp())
            }
            AttenuationType::Linear => {
                1.0 - (distance as f32 / ui_state.boundary_width as f32).powi(1)
            }
            AttenuationType::Power => {
                1.0 - (distance as f32 / ui_state.boundary_width as f32)
                    .powi(ui_state.power_order as i32)
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
}
