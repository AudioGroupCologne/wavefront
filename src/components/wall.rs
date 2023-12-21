use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::{Pos2, Rect};

use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::transformations::true_rect_from_rect;

#[derive(Component)]
pub struct Overlay;

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
}

impl WallBlock {
    pub fn new(x_min: u32, y_min: u32, x_max: u32, y_max: u32, reflection_factor: f32) -> Self {
        let rect = Rect {
            min: Pos2::new(x_min as f32, y_min as f32),
            max: Pos2::new(x_max as f32, y_max as f32),
        };

        Self {
            rect: rect,
            center: rect.center(),
            reflection_factor,
            calc_rect: rect,
        }
    }

    pub fn update_calc_rect(&mut self) {
        let calc_rect = true_rect_from_rect(self.rect);
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
    }
}
