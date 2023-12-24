use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::{Pos2, Rect};

use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::transformations::true_rect_from_rect;

#[derive(Component)]
pub enum WallResize {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
    Top,
    Right,
    Bottom,
    Left,
}

pub trait Wall {}

/// A single wall "pixel"
#[derive(Debug, Component)]
pub struct WallCell {
    pub x: u32,
    pub y: u32,
    pub reflection_factor: f32,
}

impl WallCell {
    pub fn new(x: u32, y: u32, reflection_factor: f32) -> Self {
        Self {
            x,
            y,
            reflection_factor,
        }
    }
}

impl Wall for WallCell {}

#[derive(Debug, Component)]
pub struct WallBlock {
    pub rect: Rect,
    pub center: Pos2,
    pub reflection_factor: f32,
    pub calc_rect: Rect,
    pub calc_rect_with_boundaries: Rect,
}

impl WallBlock {
    pub fn new(x_min: u32, y_min: u32, x_max: u32, y_max: u32, reflection_factor: f32) -> Self {
        let rect = Rect {
            min: Pos2::new(x_min as f32, y_min as f32),
            max: Pos2::new(x_max as f32, y_max as f32),
        };

        Self {
            rect,
            center: rect.center(),
            reflection_factor,
            calc_rect: rect,
            calc_rect_with_boundaries: rect,
        }
    }

    pub fn update_calc_rect(&mut self, e_al: u32) {
        let mut calc_rect = true_rect_from_rect(self.rect);

        self.calc_rect = Rect {
            min: Pos2::new(
                calc_rect.min.x.clamp(0., SIMULATION_WIDTH as f32 - 1.),
                calc_rect.min.y.clamp(0., SIMULATION_HEIGHT as f32 - 1.),
            ),
            max: Pos2::new(
                calc_rect.max.x.clamp(0., SIMULATION_WIDTH as f32 - 1.),
                calc_rect.max.y.clamp(0., SIMULATION_HEIGHT as f32 - 1.),
            ),
        };

        calc_rect.min.x += if calc_rect.min.x == 0. {
            1.
        } else {
            e_al as f32
        };
        calc_rect.min.y += if calc_rect.min.y == 0. {
            1.
        } else {
            e_al as f32
        };
        calc_rect.max.x += if calc_rect.max.x == (SIMULATION_WIDTH - 1) as f32 {
            (2 * e_al) as f32
        } else {
            e_al as f32
        };
        calc_rect.max.y += if calc_rect.max.y == (SIMULATION_HEIGHT - 1) as f32 {
            (2 * e_al) as f32
        } else {
            e_al as f32
        };

        self.calc_rect_with_boundaries = Rect {
            min: Pos2::new(
                calc_rect
                    .min
                    .x
                    .clamp(1., (SIMULATION_WIDTH + 2 * e_al) as f32 - 2.),
                calc_rect
                    .min
                    .y
                    .clamp(1., (SIMULATION_HEIGHT + 2 * e_al) as f32 - 2.),
            ),
            max: Pos2::new(
                calc_rect
                    .max
                    .x
                    .clamp(1., (SIMULATION_WIDTH + 2 * e_al) as f32 - 2.),
                calc_rect
                    .max
                    .y
                    .clamp(1., (SIMULATION_HEIGHT + 2 * e_al) as f32 - 2.),
            ),
        };
    }
}
