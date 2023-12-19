use bevy::prelude::*;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::WallBlock;
use crate::math::constants::*;
use crate::math::transformations::{coords_to_index, true_rect_from_rect};
use crate::render::state::{AttenuationType, UiState};

#[derive(Debug, Resource)]
pub struct Grid {
    /// full grid: [cur_bottom, cur_left, cur_top, cur_right, next_bottom, next_left, next_top, next_right, pressure]
    pub cells: Vec<f32>,
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
            delta_t: 0.001 / PROPAGATION_SPEED,
        }
    }
}

impl Grid {
    pub fn update_delta_t(&mut self, ui_state: Res<UiState>) {
        self.delta_t = ui_state.delta_l / PROPAGATION_SPEED;
    }

    pub fn update_cells(&mut self, e_al: u32) {
        self.cells = vec![
            0.;
            ((SIMULATION_WIDTH + 2 * e_al) * (SIMULATION_HEIGHT + 2 * e_al) * NUM_INDEX)
                as usize
        ];
    }

    pub fn update(&mut self, e_al: u32) {
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

    pub fn calc(&mut self, e_al: u32) {
        for x in e_al..(SIMULATION_WIDTH + e_al) {
            for y in e_al..(SIMULATION_HEIGHT + e_al) {
                self.calc_cell(
                    coords_to_index(x, y, 0, e_al),
                    self.cells[coords_to_index(x, y + 1, 2, e_al)],
                    self.cells[coords_to_index(x - 1, y, 3, e_al)],
                    self.cells[coords_to_index(x, y - 1, 0, e_al)],
                    self.cells[coords_to_index(x + 1, y, 1, e_al)],
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

    pub fn apply_sources(&mut self, ticks_since_start: u64, sources: &Query<&Source>, e_al: u32) {
        let time = self.delta_t * ticks_since_start as f32; //the cast feels wrong, but it works for now
        for source in sources.iter() {
            let calc = source.calc(time);
            let source_pos = coords_to_index(source.x + e_al, source.y + e_al, 0, e_al);
            self.cells[source_pos + 4] = calc;
            self.cells[source_pos + 5] = calc;
            self.cells[source_pos + 6] = calc;
            self.cells[source_pos + 7] = calc;
        }
    }

    pub fn apply_microphones(
        &self,
        mut microphones: Query<&mut Microphone>,
        e_al: u32,
        plot_enabled: bool,
        fft_enabled: bool,
    ) {
        if plot_enabled || fft_enabled {
            for mut mic in microphones.iter_mut() {
                let x = mic.x;
                let y = mic.y;
                let cur_time = mic.record.last().unwrap()[0] + self.delta_t as f64;

                mic.record
                    .push([cur_time, self.cells[coords_to_index(x, y, 8, e_al)] as f64]);
            }
        }
    }

    pub fn apply_walls(&mut self, walls: &Query<&WallBlock>, e_al: u32) {
        for wall in walls.iter() {
            let true_rect = wall.calc_rect;

            for x in true_rect.min.x as u32..true_rect.max.x as u32 {
                for y in true_rect.min.y as u32..true_rect.max.y as u32 {
                    let wall_index = coords_to_index(x + e_al, y + e_al, 0, e_al);
                    self.cells[wall_index + 4] = 0.;
                    self.cells[wall_index + 5] = 0.;
                    self.cells[wall_index + 6] = 0.;
                    self.cells[wall_index + 7] = 0.;
                }
            }

            for x in true_rect.min.x as u32..true_rect.max.x as u32 {
                //bottom row
                let wall_index = coords_to_index(x + e_al, true_rect.max.y as u32 + e_al, 0, e_al);
                self.cells[wall_index + 4] = wall.reflection_factor
                    * self.cells
                        [coords_to_index(x + e_al, true_rect.max.y as u32 + e_al + 1, 2, e_al)];

                //top row
                let wall_index = coords_to_index(x + e_al, true_rect.min.y as u32 + e_al, 0, e_al);
                self.cells[wall_index + 6] = wall.reflection_factor
                    * self.cells
                        [coords_to_index(x + e_al, true_rect.min.y as u32 + e_al - 1, 0, e_al)];
            }

            for y in true_rect.min.y as u32..true_rect.max.y as u32 {
                //left row
                let wall_index = coords_to_index(true_rect.min.x as u32 + e_al, y + e_al, 0, e_al);
                self.cells[wall_index + 5] = wall.reflection_factor
                    * self.cells
                        [coords_to_index(true_rect.min.x as u32 + e_al - 1, y + e_al, 3, e_al)];

                //right row
                let wall_index = coords_to_index(true_rect.max.x as u32 + e_al, y + e_al, 0, e_al);
                self.cells[wall_index + 7] = wall.reflection_factor
                    * self.cells
                        [coords_to_index(true_rect.max.x as u32 + e_al + 1, y + e_al, 1, e_al)];
            }
        }
    }

    fn calc_cell_boundary(&mut self, x: u32, y: u32, e_al: u32, attenuation_factors: &[f32; 4]) {
        let current_cell = coords_to_index(x, y, 0, e_al);
        let bottom_top = self.cells[coords_to_index(x, y + 1, 2, e_al)];
        let left_right = self.cells[coords_to_index(x - 1, y, 3, e_al)];
        let top_bottom = self.cells[coords_to_index(x, y - 1, 0, e_al)];
        let right_left = self.cells[coords_to_index(x + 1, y, 1, e_al)];

        self.cells[current_cell + 4] = 0.5
            * (-bottom_top * attenuation_factors[0]
                + left_right * attenuation_factors[1]
                + top_bottom * attenuation_factors[2]
                + right_left * attenuation_factors[3]);
        self.cells[current_cell + 5] = 0.5
            * (bottom_top * attenuation_factors[0] - left_right * attenuation_factors[1]
                + top_bottom * attenuation_factors[2]
                + right_left * attenuation_factors[3]);
        self.cells[current_cell + 6] = 0.5
            * (bottom_top * attenuation_factors[0] + left_right * attenuation_factors[1]
                - top_bottom * attenuation_factors[2]
                + right_left * attenuation_factors[3]);
        self.cells[current_cell + 7] = 0.5
            * (bottom_top * attenuation_factors[0]
                + left_right * attenuation_factors[1]
                + top_bottom * attenuation_factors[2]
                - right_left * attenuation_factors[3]);
    }

    pub fn apply_boundaries(&mut self, ui_state: Res<UiState>) {
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
