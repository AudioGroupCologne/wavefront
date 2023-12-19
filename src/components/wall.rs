use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::{Pos2, Rect};

use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::transformations::true_rect_from_rect;

#[derive(Component)]
pub struct CornerResize;

/// A wall component containing the coordinates of the corresponding cell in the grid
#[derive(Debug)]
pub struct WallCell {
    pub x: u32,
    pub y: u32,
}

impl WallCell {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Component)]
pub struct WallBlock {
    pub rect: Rect,
    pub center: Pos2,
    pub reflection_factor: f32,
    pub calc_rect: Rect,
}

impl WallBlock {
    pub fn new(x: u32, y: u32, reflection_factor: f32) -> Self {
        Self {
            rect: Rect {
                min: Pos2::new(x as f32, y as f32),
                max: Pos2::new(x as f32, y as f32),
            },
            center: Pos2 {
                x: x as f32,
                y: y as f32,
            },
            reflection_factor,
            calc_rect: Rect {
                min: Pos2::new(x as f32, y as f32),
                max: Pos2::new(x as f32, y as f32),
            },
        }
    }

    pub fn update_calc_rect(&mut self) {
        let calc_rect = true_rect_from_rect(self.rect);
        self.calc_rect = Rect {
            min: Pos2::new(
                calc_rect.min.x.clamp(0., SIMULATION_WIDTH as f32),
                calc_rect.min.y.clamp(0., SIMULATION_HEIGHT as f32),
            ),
            max: Pos2::new(
                calc_rect.max.x.clamp(0., SIMULATION_WIDTH as f32),
                calc_rect.max.y.clamp(0., SIMULATION_HEIGHT as f32),
            ),
        };
    }
}
