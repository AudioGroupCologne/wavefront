use bevy::prelude::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::states::Overlay;
use crate::components::wall::{Wall, WallType};
use crate::math::constants::*;
use crate::math::transformations::{coords_to_index, index_to_coords};
use crate::ui::state::{AttenuationType, UiState};

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub cur_bottom: f32,
    pub cur_left: f32,
    pub cur_top: f32,
    pub cur_right: f32,
    pub next_bottom: f32,
    pub next_left: f32,
    pub next_top: f32,
    pub next_right: f32,
    pub pressure: f32,
}

impl Cell {
    fn update(&mut self) {
        self.cur_bottom = self.next_bottom;
        self.cur_left = self.next_left;
        self.cur_top = self.next_top;
        self.cur_right = self.next_right;

        self.pressure = 0.5 * (self.cur_bottom + self.cur_left + self.cur_top + self.cur_right);
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            cur_bottom: 0.,
            cur_left: 0.,
            cur_top: 0.,
            cur_right: 0.,
            next_bottom: 0.,
            next_left: 0.,
            next_top: 0.,
            next_right: 0.,
            pressure: 0.,
        }
    }
}

#[derive(Debug, Resource)]
pub struct Grid {
    /// Grid cells
    pub cells: Vec<Cell>,
    /// Delta s in seconds
    pub delta_t: f32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cells: vec![
                Cell::default();
                ((SIMULATION_WIDTH + 2 * E_AL) * (SIMULATION_HEIGHT + 2 * E_AL)) as usize
            ],
            delta_t: 0.001 / PROPAGATION_SPEED,
        }
    }
}

impl Grid {
    pub fn update_delta_t(&mut self, ui_state: &UiState) {
        self.delta_t = ui_state.delta_l / PROPAGATION_SPEED;
    }

    pub fn reset_cells(&mut self, e_al: u32) {
        self.cells = vec![
            Cell::default();
            ((SIMULATION_WIDTH + 2 * e_al) * (SIMULATION_HEIGHT + 2 * e_al)) as usize
        ];
    }

    pub fn update_cells(&mut self) {
        self.cells.par_iter_mut().for_each(|cell| {
            cell.update();
        });
    }

    pub fn calc_cells(&mut self, e_al: u32) {
        for x in e_al..(SIMULATION_WIDTH + e_al) {
            for y in e_al..(SIMULATION_HEIGHT + e_al) {
                let current_cell_index = coords_to_index(x, y, e_al);
                let bottom_cell = self.cells[coords_to_index(x, y + 1, e_al)];
                let left_cell = self.cells[coords_to_index(x - 1, y, e_al)];
                let top_cell = self.cells[coords_to_index(x, y - 1, e_al)];
                let right_cell = self.cells[coords_to_index(x + 1, y, e_al)];

                self.cells[current_cell_index].next_bottom = 0.5
                    * (-bottom_cell.cur_top
                        + left_cell.cur_right
                        + top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_left = 0.5
                    * (bottom_cell.cur_top - left_cell.cur_right
                        + top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_top = 0.5
                    * (bottom_cell.cur_top + left_cell.cur_right - top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_right = 0.5
                    * (bottom_cell.cur_top + left_cell.cur_right + top_cell.cur_bottom
                        - right_cell.cur_left);
            }
        }

        // this does not work, because we cant access the cells vec insinde the closure
        // exclude boundary??
        // self.cells
        //     .par_iter_mut()
        //     .enumerate()
        //     .for_each(|(index, cell)| {
        //         let (x, y) = index_to_coords(index as u32, e_al);

        //         // we cant access the same vec over which we are mutable iterating,
        //         // maybe store the next values in a separate vec and then update the current values?
        //         let bottom_cell = self.cells[coords_to_index(x, y + 1, e_al)].clone();
        //         let left_cell = self.cells[coords_to_index(x - 1, y, e_al)];
        //         let top_cell = self.cells[coords_to_index(x, y - 1, e_al)];
        //         let right_cell = self.cells[coords_to_index(x + 1, y, e_al)];

        //         cell.next_bottom = 0.5
        //             * (-bottom_cell.cur_top
        //                 + left_cell.cur_right
        //                 + top_cell.cur_bottom
        //                 + right_cell.cur_left);
        //         cell.next_left = 0.5
        //             * (bottom_cell.cur_top - left_cell.cur_right
        //                 + top_cell.cur_bottom
        //                 + right_cell.cur_left);
        //         cell.next_top = 0.5
        //             * (bottom_cell.cur_top + left_cell.cur_right - top_cell.cur_bottom
        //                 + right_cell.cur_left);
        //         cell.next_right = 0.5
        //             * (bottom_cell.cur_top + left_cell.cur_right + top_cell.cur_bottom
        //                 - right_cell.cur_left);
        //     });
    }

    pub fn apply_sources(&mut self, ticks_since_start: u64, sources: &Query<&Source>, e_al: u32) {
        let time = self.delta_t * ticks_since_start as f32; //the cast feels wrong, but it works for now
        for source in sources.iter() {
            let calc = source.calc(time);
            let source_pos = coords_to_index(source.x + e_al, source.y + e_al, e_al);
            self.cells[source_pos].next_bottom = calc;
            self.cells[source_pos].next_left = calc;
            self.cells[source_pos].next_top = calc;
            self.cells[source_pos].next_right = calc;
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
                    self.cells[coords_to_index(x, y, ui_state.e_al)].pressure as f64,
                ]);
            }
        }
    }

    pub fn apply_walls(&mut self, walls: &Query<&Wall, Without<Overlay>>, e_al: u32) {
        for wall in walls.iter() {
            match &wall.wall_type {
                WallType::Rectangle => {
                    if !wall.hollow {
                        for x in wall.calc_rect.min.x..=wall.calc_rect.max.x {
                            for y in wall.calc_rect.min.y..=wall.calc_rect.max.y {
                                let wall_index = coords_to_index(x, y, e_al);
                                self.cells[wall_index].next_bottom = 0.;
                                self.cells[wall_index].next_left = 0.;
                                self.cells[wall_index].next_top = 0.;
                                self.cells[wall_index].next_right = 0.;
                            }
                        }
                    }

                    for x in wall.calc_rect.min.x..=wall.calc_rect.max.x {
                        //bottom row
                        let wall_index = coords_to_index(x, wall.calc_rect.max.y, e_al);
                        //outer reflection
                        self.cells[wall_index].next_bottom = wall.reflection_factor
                            * self.cells[coords_to_index(x, wall.calc_rect.max.y + 1, e_al)]
                                .cur_top;
                        // inner reflection
                        self.cells[wall_index].next_top = wall.reflection_factor
                            * self.cells[coords_to_index(x, wall.calc_rect.max.y - 1, e_al)]
                                .cur_bottom;

                        //top row
                        let wall_index = coords_to_index(x, wall.calc_rect.min.y as u32, e_al);
                        //outer reflection
                        self.cells[wall_index].next_top = wall.reflection_factor
                            * self.cells[coords_to_index(x, wall.calc_rect.min.y - 1, e_al)]
                                .cur_bottom;
                        // inner reflection
                        self.cells[wall_index].next_bottom = wall.reflection_factor
                            * self.cells[coords_to_index(x, wall.calc_rect.min.y + 1, e_al)]
                                .cur_top;
                    }

                    for y in wall.calc_rect.min.y as u32..=wall.calc_rect.max.y as u32 {
                        //left row
                        let wall_index = coords_to_index(wall.calc_rect.min.x as u32, y, e_al);
                        //outer reflection
                        self.cells[wall_index].next_left = wall.reflection_factor
                            * self.cells[coords_to_index(wall.calc_rect.min.x - 1, y, e_al)]
                                .cur_right;
                        // inner reflection
                        self.cells[wall_index].next_right = wall.reflection_factor
                            * self.cells[coords_to_index(wall.calc_rect.min.x + 1, y, e_al)]
                                .cur_left;

                        //right row
                        let wall_index = coords_to_index(wall.calc_rect.max.x as u32, y, e_al);
                        //outer reflection
                        self.cells[wall_index].next_right = wall.reflection_factor
                            * self.cells[coords_to_index(wall.calc_rect.max.x + 1, y, e_al)]
                                .cur_left;
                        // inner reflection
                        self.cells[wall_index].next_left = wall.reflection_factor
                            * self.cells[coords_to_index(wall.calc_rect.max.x - 1, y, e_al)]
                                .cur_right;
                    }
                }
                WallType::Circle => todo!(),
            }
        }
    }

    pub fn apply_boundaries(&mut self, ui_state: &UiState) {
        let b = (ui_state.e_al * ui_state.e_al) as f32 / ui_state.epsilon.ln();
        // going in 'rings' from outer to inner
        // every ring shares an attenuation factor
        for r in 1..ui_state.e_al {
            // there was a '?' in front of ui_state, that's not needed right?
            // also distance could be just r -> need to redo att_fac calcs
            let attenuation_factor = Grid::attenuation_factor(ui_state, ui_state.e_al - r, b);

            // bottom
            for x in r..(SIMULATION_WIDTH + 2 * ui_state.e_al - r) {
                let y = SIMULATION_HEIGHT + 2 * ui_state.e_al - r - 1;
                let current_cell_index = coords_to_index(x, y, ui_state.e_al);
                let bottom_cell = self.cells[coords_to_index(x, y + 1, ui_state.e_al)];
                let left_cell = self.cells[coords_to_index(x - 1, y, ui_state.e_al)];
                let top_cell = self.cells[coords_to_index(x, y - 1, ui_state.e_al)];
                let right_cell = self.cells[coords_to_index(x + 1, y, ui_state.e_al)];

                self.cells[current_cell_index].next_bottom = 0.5
                    * (-bottom_cell.cur_top
                        + left_cell.cur_right
                        + attenuation_factor * top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_left = 0.5
                    * (bottom_cell.cur_top - left_cell.cur_right
                        + attenuation_factor * top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_top = 0.5
                    * (bottom_cell.cur_top + left_cell.cur_right
                        - attenuation_factor * top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_right = 0.5
                    * (bottom_cell.cur_top
                        + left_cell.cur_right
                        + attenuation_factor * top_cell.cur_bottom
                        - right_cell.cur_left);
            }
            // left
            for y in r..(SIMULATION_HEIGHT + 2 * ui_state.e_al - r) {
                let x = r;
                let current_cell_index = coords_to_index(x, y, ui_state.e_al);
                let bottom_cell = self.cells[coords_to_index(x, y + 1, ui_state.e_al)];
                let left_cell = self.cells[coords_to_index(x - 1, y, ui_state.e_al)];
                let top_cell = self.cells[coords_to_index(x, y - 1, ui_state.e_al)];
                let right_cell = self.cells[coords_to_index(x + 1, y, ui_state.e_al)];

                self.cells[current_cell_index].next_bottom = 0.5
                    * (-bottom_cell.cur_top
                        + left_cell.cur_right
                        + top_cell.cur_bottom
                        + attenuation_factor * right_cell.cur_left);
                self.cells[current_cell_index].next_left = 0.5
                    * (bottom_cell.cur_top - left_cell.cur_right
                        + top_cell.cur_bottom
                        + attenuation_factor * right_cell.cur_left);
                self.cells[current_cell_index].next_top = 0.5
                    * (bottom_cell.cur_top + left_cell.cur_right - top_cell.cur_bottom
                        + attenuation_factor * right_cell.cur_left);
                self.cells[current_cell_index].next_right = 0.5
                    * (bottom_cell.cur_top + left_cell.cur_right + top_cell.cur_bottom
                        - attenuation_factor * right_cell.cur_left);
            }
            // top
            for x in r..(SIMULATION_WIDTH + 2 * ui_state.e_al - r) {
                let y = r;
                let current_cell_index = coords_to_index(x, y, ui_state.e_al);
                let bottom_cell = self.cells[coords_to_index(x, y + 1, ui_state.e_al)];
                let left_cell = self.cells[coords_to_index(x - 1, y, ui_state.e_al)];
                let top_cell = self.cells[coords_to_index(x, y - 1, ui_state.e_al)];
                let right_cell = self.cells[coords_to_index(x + 1, y, ui_state.e_al)];

                self.cells[current_cell_index].next_bottom = 0.5
                    * (attenuation_factor * -bottom_cell.cur_top
                        + left_cell.cur_right
                        + top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_left = 0.5
                    * (attenuation_factor * bottom_cell.cur_top - left_cell.cur_right
                        + top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_top = 0.5
                    * (attenuation_factor * bottom_cell.cur_top + left_cell.cur_right
                        - top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_right = 0.5
                    * (attenuation_factor * bottom_cell.cur_top
                        + left_cell.cur_right
                        + top_cell.cur_bottom
                        - right_cell.cur_left);
            }
            // right
            for y in r..(SIMULATION_HEIGHT + 2 * ui_state.e_al - r) {
                let x = SIMULATION_WIDTH + 2 * ui_state.e_al - r - 1;
                let current_cell_index = coords_to_index(x, y, ui_state.e_al);
                let bottom_cell = self.cells[coords_to_index(x, y + 1, ui_state.e_al)];
                let left_cell = self.cells[coords_to_index(x - 1, y, ui_state.e_al)];
                let top_cell = self.cells[coords_to_index(x, y - 1, ui_state.e_al)];
                let right_cell = self.cells[coords_to_index(x + 1, y, ui_state.e_al)];

                self.cells[current_cell_index].next_bottom = 0.5
                    * (-bottom_cell.cur_top
                        + attenuation_factor * left_cell.cur_right
                        + top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_left = 0.5
                    * (bottom_cell.cur_top - attenuation_factor * left_cell.cur_right
                        + top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_top = 0.5
                    * (bottom_cell.cur_top + attenuation_factor * left_cell.cur_right
                        - top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[current_cell_index].next_right = 0.5
                    * (bottom_cell.cur_top
                        + attenuation_factor * left_cell.cur_right
                        + top_cell.cur_bottom
                        - right_cell.cur_left);
            }
        }
    }

    fn attenuation_factor(ui_state: &UiState, distance: u32, b: f32) -> f32 {
        match ui_state.at_type {
            AttenuationType::OriginalOneWay => {
                1.0 - ((1. + ui_state.epsilon) - ((distance * distance) as f32 / b).exp())
            }
            AttenuationType::Linear => 1.0 - (distance as f32 / ui_state.e_al as f32).powi(1),
            AttenuationType::Power => {
                1.0 - (distance as f32 / ui_state.e_al as f32).powi(ui_state.power_order as i32)
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
