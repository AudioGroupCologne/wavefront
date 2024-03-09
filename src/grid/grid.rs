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
                ((SIMULATION_WIDTH + 2 * E_AL) * (SIMULATION_HEIGHT + 2 * E_AL))
                    as usize
            ],
            next_cells: vec![
                Cell::default();
                ((SIMULATION_WIDTH + 2 * E_AL) * (SIMULATION_HEIGHT + 2 * E_AL))
                    as usize
            ],
            pressure: vec![
                0_f32;
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
        self.cur_cells = vec![
            Cell::default();
            ((SIMULATION_WIDTH + 2 * e_al) * (SIMULATION_HEIGHT + 2 * e_al))
                as usize
        ];
        self.next_cells = vec![
            Cell::default();
            ((SIMULATION_WIDTH + 2 * e_al) * (SIMULATION_HEIGHT + 2 * e_al))
                as usize
        ];
        self.pressure =
            vec![0_f32; ((SIMULATION_WIDTH + 2 * E_AL) * (SIMULATION_HEIGHT + 2 * E_AL)) as usize];
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

    pub fn calc_cells(&mut self, e_al: u32) {
        self.next_cells
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, cell)| {
                let (x, y) = index_to_coords(index as u32, e_al);
                // if pixel is in sim region
                if x >= e_al
                    && x < SIMULATION_WIDTH + e_al
                    && y >= e_al
                    && y < SIMULATION_HEIGHT + e_al
                {
                    let bottom_cell = self.cur_cells[coords_to_index(x, y + 1, e_al)];
                    let left_cell = self.cur_cells[coords_to_index(x - 1, y, e_al)];
                    let top_cell = self.cur_cells[coords_to_index(x, y - 1, e_al)];
                    let right_cell = self.cur_cells[coords_to_index(x + 1, y, e_al)];

                    cell.bottom = 0.5
                        * (-bottom_cell.top + left_cell.right + top_cell.bottom + right_cell.left);
                    cell.left = 0.5
                        * (bottom_cell.top - left_cell.right + top_cell.bottom + right_cell.left);
                    cell.top = 0.5
                        * (bottom_cell.top + left_cell.right - top_cell.bottom + right_cell.left);
                    cell.right = 0.5
                        * (bottom_cell.top + left_cell.right + top_cell.bottom - right_cell.left);
                }
            });
    }

    pub fn apply_sources(&mut self, ticks_since_start: u64, sources: &Query<&Source>, e_al: u32) {
        let time = self.delta_t * ticks_since_start as f32; //the cast feels wrong, but it works for now
        for source in sources.iter() {
            let calc = source.calc(time);
            let source_pos = coords_to_index(source.x + e_al, source.y + e_al, e_al);
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
                    self.pressure[coords_to_index(x, y, ui_state.e_al)] as f64,
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
                                self.next_cells[wall_index].bottom = 0.;
                                self.next_cells[wall_index].left = 0.;
                                self.next_cells[wall_index].top = 0.;
                                self.next_cells[wall_index].right = 0.;
                            }
                        }
                    }

                    for x in wall.calc_rect.min.x..=wall.calc_rect.max.x {
                        //bottom row
                        let wall_index = coords_to_index(x, wall.calc_rect.max.y, e_al);
                        //outer reflection
                        self.next_cells[wall_index].bottom = wall.reflection_factor
                            * self.cur_cells[coords_to_index(x, wall.calc_rect.max.y + 1, e_al)]
                                .top;
                        // inner reflection
                        self.next_cells[wall_index].top = wall.reflection_factor
                            * self.cur_cells[coords_to_index(x, wall.calc_rect.max.y - 1, e_al)]
                                .bottom;

                        //top row
                        let wall_index = coords_to_index(x, wall.calc_rect.min.y as u32, e_al);
                        //outer reflection
                        self.next_cells[wall_index].top = wall.reflection_factor
                            * self.cur_cells[coords_to_index(x, wall.calc_rect.min.y - 1, e_al)]
                                .bottom;
                        // inner reflection
                        self.next_cells[wall_index].bottom = wall.reflection_factor
                            * self.cur_cells[coords_to_index(x, wall.calc_rect.min.y + 1, e_al)]
                                .top;
                    }

                    for y in wall.calc_rect.min.y as u32..=wall.calc_rect.max.y as u32 {
                        //left row
                        let wall_index = coords_to_index(wall.calc_rect.min.x as u32, y, e_al);
                        //outer reflection
                        self.next_cells[wall_index].left = wall.reflection_factor
                            * self.cur_cells[coords_to_index(wall.calc_rect.min.x - 1, y, e_al)]
                                .right;
                        // inner reflection
                        self.next_cells[wall_index].right = wall.reflection_factor
                            * self.cur_cells[coords_to_index(wall.calc_rect.min.x + 1, y, e_al)]
                                .left;

                        //right row
                        let wall_index = coords_to_index(wall.calc_rect.max.x as u32, y, e_al);
                        //outer reflection
                        self.next_cells[wall_index].right = wall.reflection_factor
                            * self.cur_cells[coords_to_index(wall.calc_rect.max.x + 1, y, e_al)]
                                .left;
                        // inner reflection
                        self.next_cells[wall_index].left = wall.reflection_factor
                            * self.cur_cells[coords_to_index(wall.calc_rect.max.x - 1, y, e_al)]
                                .right;
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
                let bottom_cell = self.cur_cells[coords_to_index(x, y + 1, ui_state.e_al)];
                let left_cell = self.cur_cells[coords_to_index(x - 1, y, ui_state.e_al)];
                let top_cell = self.cur_cells[coords_to_index(x, y - 1, ui_state.e_al)];
                let right_cell = self.cur_cells[coords_to_index(x + 1, y, ui_state.e_al)];

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
            for y in r..(SIMULATION_HEIGHT + 2 * ui_state.e_al - r) {
                let x = r;
                let current_cell_index = coords_to_index(x, y, ui_state.e_al);
                let bottom_cell = self.cur_cells[coords_to_index(x, y + 1, ui_state.e_al)];
                let left_cell = self.cur_cells[coords_to_index(x - 1, y, ui_state.e_al)];
                let top_cell = self.cur_cells[coords_to_index(x, y - 1, ui_state.e_al)];
                let right_cell = self.cur_cells[coords_to_index(x + 1, y, ui_state.e_al)];

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
            for x in r..(SIMULATION_WIDTH + 2 * ui_state.e_al - r) {
                let y = r;
                let current_cell_index = coords_to_index(x, y, ui_state.e_al);
                let bottom_cell = self.cur_cells[coords_to_index(x, y + 1, ui_state.e_al)];
                let left_cell = self.cur_cells[coords_to_index(x - 1, y, ui_state.e_al)];
                let top_cell = self.cur_cells[coords_to_index(x, y - 1, ui_state.e_al)];
                let right_cell = self.cur_cells[coords_to_index(x + 1, y, ui_state.e_al)];

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
            for y in r..(SIMULATION_HEIGHT + 2 * ui_state.e_al - r) {
                let x = SIMULATION_WIDTH + 2 * ui_state.e_al - r - 1;
                let current_cell_index = coords_to_index(x, y, ui_state.e_al);
                let bottom_cell = self.cur_cells[coords_to_index(x, y + 1, ui_state.e_al)];
                let left_cell = self.cur_cells[coords_to_index(x - 1, y, ui_state.e_al)];
                let top_cell = self.cur_cells[coords_to_index(x, y - 1, ui_state.e_al)];
                let right_cell = self.cur_cells[coords_to_index(x + 1, y, ui_state.e_al)];

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
