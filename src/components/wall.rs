use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::{Pos2, Rect};

#[derive(Component)]
pub struct CornerResize;

pub trait Wall{}

/// A single wall "pixel"
#[derive(Debug, Component)]
pub struct WallCell {
    pub x: u32,
    pub y: u32,
    pub reflection_factor: f32,
}

impl WallCell {
    pub fn new(x: u32, y: u32, reflection_factor: f32) -> Self {
        Self { x, y, reflection_factor }
    }
}

impl Wall for WallCell {}

#[derive(Debug, Component)]
pub struct WallBlock {
    pub rect: Rect,
    pub center: Pos2,
    pub reflection_factor: f32,
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
        }
    }

    pub fn update(&mut self) {
        // really horrible update, need to think of sth better
        self.center = self.rect.center();
    }
}
