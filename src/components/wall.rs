use std::cmp::Ordering;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::rect::WRect;

#[derive(Component)]
pub enum WResize {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
    // Top,
    // Right,
    // Bottom,
    // Left,
    Radius,
}

pub trait Wall {
    fn get_center(&self) -> UVec2;

    fn get_resize_point(&self, resize_type: WResize) -> UVec2;

    fn contains(&self, x: u32, y: u32) -> bool;

    fn edge_contains(&self, x: u32, y: u32) -> bool;

    /// If width or height equals one, the wall can be deleted
    fn is_deletable(&self) -> bool;

    fn set_center(&mut self, x: u32, y: u32);
}

#[derive(Component)]
pub struct RectWall {
    // between 0 and SIM_WIDTH + 2 * boundary_width
    pub rect: WRect,
    pub is_hollow: bool,
    pub reflection_factor: f32,
    pub id: usize,
}

impl Wall for RectWall {
    fn get_center(&self) -> UVec2 {
        todo!()
    }

    fn get_resize_point(&self, resize_type: WResize) -> UVec2 {
        todo!()
    }

    fn contains(&self, x: u32, y: u32) -> bool {
        todo!()
    }

    fn edge_contains(&self, x: u32, y: u32) -> bool {
        todo!()
    }

    fn is_deletable(&self) -> bool {
        todo!()
    }

    fn set_center(&mut self, x: u32, y: u32) {
        todo!()
    }
}

impl RectWall {
    pub fn new(
        x0: u32,
        y0: u32,
        x1: u32,
        y1: u32,
        is_hollow: bool,
        reflection_factor: f32,
    ) -> Self {
        todo!()
    }

    pub fn set_top_left(&mut self, x: u32, y: u32) {
        todo!()
    }

    pub fn set_top_right(&mut self, x: u32, y: u32) {
        todo!()
    }

    pub fn set_bottom_left(&mut self, x: u32, y: u32) {
        todo!()
    }

    pub fn set_bottom_right(&mut self, x: u32, y: u32) {
        todo!()
    }
}

#[derive(Component)]
pub struct CircWall {
    pub center: UVec2,
    /// Radius excludes center point
    pub radius: u32,
    pub is_hollow: bool,
    pub reflection_factor: f32,
    pub id: usize,
}

impl Wall for CircWall {
    fn get_center(&self) -> UVec2 {
        todo!()
    }

    fn get_resize_point(&self, resize_type: WResize) -> UVec2 {
        todo!()
    }

    fn contains(&self, x: u32, y: u32) -> bool {
        todo!()
    }

    fn edge_contains(&self, x: u32, y: u32) -> bool {
        todo!()
    }

    fn is_deletable(&self) -> bool {
        todo!()
    }

    fn set_center(&mut self, x: u32, y: u32) {
        todo!()
    }
}

impl CircWall {
    pub fn new(x: u32, y: u32, radius: u32, is_hollow: bool, reflection_factor: f32) -> Self {
        todo!()
    }

    pub fn set_radius() {}
}
