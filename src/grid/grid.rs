use bevy::prelude::*;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::states::Overlay;
use crate::components::wall::WallBlock;
use crate::math::constants::*;
use crate::math::transformations::coords_to_index;
use crate::render::state::{AttenuationType, UiState};

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
                let bottom_cell = self.cells[coords_to_index(x, y + 1, e_al)];
                let left_cell = self.cells[coords_to_index(x - 1, y, e_al)];
                let top_cell = self.cells[coords_to_index(x, y - 1, e_al)];
                let right_cell = self.cells[coords_to_index(x + 1, y, e_al)];

                self.cells[coords_to_index(x, y, e_al)].next_bottom = 0.5
                    * (-bottom_cell.cur_top
                        + left_cell.cur_right
                        + top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[coords_to_index(x, y, e_al)].next_left = 0.5
                    * (bottom_cell.cur_top - left_cell.cur_right
                        + top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[coords_to_index(x, y, e_al)].next_top = 0.5
                    * (bottom_cell.cur_top + left_cell.cur_right - top_cell.cur_bottom
                        + right_cell.cur_left);
                self.cells[coords_to_index(x, y, e_al)].next_right = 0.5
                    * (bottom_cell.cur_top + left_cell.cur_right + top_cell.cur_bottom
                        - right_cell.cur_left);
            }
        }

        // index to coord, and back to index
        // exclude boundary??
        // self.cells
        //     .par_iter_mut()
        //     .enumerate()
        //     .for_each(|(index, cell)| {
        //         cell.update();
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

    pub fn apply_walls(&mut self, wallblocks: &Query<&WallBlock, Without<Overlay>>, e_al: u32) {
        for wall in wallblocks.iter() {
            let true_rect = wall.calc_rect_with_boundaries;

            for x in true_rect.min.x as u32..=true_rect.max.x as u32 {
                for y in true_rect.min.y as u32..=true_rect.max.y as u32 {
                    let wall_index = coords_to_index(x, y, e_al);
                    self.cells[wall_index].next_bottom = 0.;
                    self.cells[wall_index].next_left = 0.;
                    self.cells[wall_index].next_top = 0.;
                    self.cells[wall_index].next_right = 0.;
                }
            }

            for x in true_rect.min.x as u32..=true_rect.max.x as u32 {
                //bottom row
                let wall_index = coords_to_index(x, true_rect.max.y as u32, e_al);
                self.cells[wall_index].next_bottom = wall.reflection_factor
                    * self.cells[coords_to_index(x, true_rect.max.y as u32 + 1, e_al)].cur_top;

                //top row
                let wall_index = coords_to_index(x, true_rect.min.y as u32, e_al);
                self.cells[wall_index].next_top = wall.reflection_factor
                    * self.cells[coords_to_index(x, true_rect.min.y as u32 - 1, e_al)].cur_bottom;
            }

            for y in true_rect.min.y as u32..=true_rect.max.y as u32 {
                //left row
                let wall_index = coords_to_index(true_rect.min.x as u32, y, e_al);
                self.cells[wall_index].next_left = wall.reflection_factor
                    * self.cells[coords_to_index(true_rect.min.x as u32 - 1, y, e_al)].cur_right;

                //right row
                let wall_index = coords_to_index(true_rect.max.x as u32, y, e_al);
                self.cells[wall_index].next_right = wall.reflection_factor
                    * self.cells[coords_to_index(true_rect.max.x as u32 + 1, y, e_al)].cur_left;
            }
        }
    }

    fn calc_cell_boundary(&mut self, x: u32, y: u32, e_al: u32, attenuation_factors: &[f32; 4]) {
        let current_cell = coords_to_index(x, y, e_al);
        let bottom_top = self.cells[coords_to_index(x, y + 1, e_al)].cur_top;
        let left_right = self.cells[coords_to_index(x - 1, y, e_al)].cur_right;
        let top_bottom = self.cells[coords_to_index(x, y - 1, e_al)].cur_bottom;
        let right_left = self.cells[coords_to_index(x + 1, y, e_al)].cur_left;

        self.cells[current_cell].next_bottom = 0.5
            * (-bottom_top * attenuation_factors[0]
                + left_right * attenuation_factors[1]
                + top_bottom * attenuation_factors[2]
                + right_left * attenuation_factors[3]);
        self.cells[current_cell].next_left = 0.5
            * (bottom_top * attenuation_factors[0] - left_right * attenuation_factors[1]
                + top_bottom * attenuation_factors[2]
                + right_left * attenuation_factors[3]);
        self.cells[current_cell].next_top = 0.5
            * (bottom_top * attenuation_factors[0] + left_right * attenuation_factors[1]
                - top_bottom * attenuation_factors[2]
                + right_left * attenuation_factors[3]);
        self.cells[current_cell].next_right = 0.5
            * (bottom_top * attenuation_factors[0]
                + left_right * attenuation_factors[1]
                + top_bottom * attenuation_factors[2]
                - right_left * attenuation_factors[3]);
    }

    pub fn apply_boundaries(&mut self, ui_state: &UiState) {
        let b = (ui_state.e_al * ui_state.e_al) as f32 / ui_state.epsilon.ln();

        //Left
        for x in 1..ui_state.e_al {
            for y in ui_state.e_al..(SIMULATION_HEIGHT + ui_state.e_al) {
                let attenuation_factor = Grid::attenuation_factor(&ui_state, ui_state.e_al - x, b);

                self.calc_cell_boundary(x, y, ui_state.e_al, &[1., 1., 1., attenuation_factor]);
            }
        }
        //Top
        for x in ui_state.e_al..(SIMULATION_WIDTH + ui_state.e_al) {
            for y in 1..ui_state.e_al {
                let attenuation_factor = Grid::attenuation_factor(&ui_state, ui_state.e_al - y, b);

                self.calc_cell_boundary(x, y, ui_state.e_al, &[attenuation_factor, 1., 1., 1.]);
            }
        }
        //Right
        for x in (SIMULATION_WIDTH + ui_state.e_al)..(SIMULATION_WIDTH + 2 * ui_state.e_al - 1) {
            for y in ui_state.e_al..(SIMULATION_HEIGHT + ui_state.e_al) {
                let attenuation_factor =
                    Grid::attenuation_factor(&ui_state, x - (SIMULATION_WIDTH + ui_state.e_al), b);

                self.calc_cell_boundary(x, y, ui_state.e_al, &[1., attenuation_factor, 1., 1.]);
            }
        }
        //Bottom
        for x in ui_state.e_al..(SIMULATION_WIDTH + ui_state.e_al) {
            for y in
                (SIMULATION_HEIGHT + ui_state.e_al)..(SIMULATION_HEIGHT + 2 * ui_state.e_al - 1)
            {
                let attenuation_factor =
                    Grid::attenuation_factor(&ui_state, y - (SIMULATION_HEIGHT + ui_state.e_al), b);

                self.calc_cell_boundary(x, y, ui_state.e_al, &[1., 1., attenuation_factor, 1.]);
            }
        }
        //LeftTop
        for x in 1..ui_state.e_al {
            for y in 1..ui_state.e_al {
                let attenuation_factor_left =
                    Grid::attenuation_factor(&ui_state, ui_state.e_al - x, b);

                let attenuation_factor_top =
                    Grid::attenuation_factor(&ui_state, ui_state.e_al - y, b);

                self.calc_cell_boundary(
                    x,
                    y,
                    ui_state.e_al,
                    &[attenuation_factor_top, 1., 1., attenuation_factor_left],
                );
            }
        }
        //RightTop
        for x in (SIMULATION_WIDTH + ui_state.e_al)..(SIMULATION_WIDTH + 2 * ui_state.e_al - 1) {
            for y in 1..ui_state.e_al {
                let attenuation_factor_right =
                    Grid::attenuation_factor(&ui_state, x - (SIMULATION_WIDTH + ui_state.e_al), b);

                let attenuation_factor_top =
                    Grid::attenuation_factor(&ui_state, ui_state.e_al - y, b);

                self.calc_cell_boundary(
                    x,
                    y,
                    ui_state.e_al,
                    &[attenuation_factor_top, attenuation_factor_right, 1., 1.],
                );
            }
        }
        //RightBottom
        for x in (SIMULATION_WIDTH + ui_state.e_al)..(SIMULATION_WIDTH + 2 * ui_state.e_al - 1) {
            for y in
                (SIMULATION_HEIGHT + ui_state.e_al)..(SIMULATION_HEIGHT + 2 * ui_state.e_al - 1)
            {
                let attenuation_factor_right =
                    Grid::attenuation_factor(&ui_state, x - (SIMULATION_WIDTH + ui_state.e_al), b);

                let attenuation_factor_bottom =
                    Grid::attenuation_factor(&ui_state, y - (SIMULATION_HEIGHT + ui_state.e_al), b);

                self.calc_cell_boundary(
                    x,
                    y,
                    ui_state.e_al,
                    &[1., attenuation_factor_right, attenuation_factor_bottom, 1.],
                );
            }
        }
        //LeftBottom
        for x in 1..ui_state.e_al {
            for y in
                (SIMULATION_HEIGHT + ui_state.e_al)..(SIMULATION_HEIGHT + 2 * ui_state.e_al - 1)
            {
                let attenuation_factor_left =
                    Grid::attenuation_factor(&ui_state, ui_state.e_al - x, b);

                let attenuation_factor_bottom =
                    Grid::attenuation_factor(&ui_state, y - (SIMULATION_HEIGHT + ui_state.e_al), b);

                self.calc_cell_boundary(
                    x,
                    y,
                    ui_state.e_al,
                    &[1., 1., attenuation_factor_bottom, attenuation_factor_left],
                );
            }
        }
    }

    fn attenuation_factor(ui_state: &UiState, distance: u32, b: f32) -> f32 {
        match ui_state.at_type {
            AttenuationType::OriginalOneWay => {
                1.0 - ((1. + ui_state.epsilon) - ((distance * distance) as f32 / b).exp())
            }
            AttenuationType::Linear => 1.0 - ((distance) as f32 / ui_state.e_al as f32).powi(1),
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
