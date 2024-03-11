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

pub trait Wall: Sync + Send {
    fn get_resize_point(&self, resize_type: WResize) -> UVec2;

    fn contains(&self, x: u32, y: u32) -> bool;

    fn edge_contains(&self, x: u32, y: u32) -> bool;

    /// If width or height equals one, the wall can be deleted
    fn is_deletable(&self) -> bool;

    fn set_center(&mut self, x: u32, y: u32);

    fn get_center(&self) -> UVec2;

    fn get_reflection_factor(&self) -> f32;
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct RectWall {
    // between 0 and SIM_WIDTH
    // between 0 and SIM_HEIGHT
    pub rect: WRect,
    pub is_hollow: bool,
    pub reflection_factor: f32,
    pub id: usize,
}

impl Wall for RectWall {
    fn get_center(&self) -> UVec2 {
        self.rect.center()
    }

    fn get_resize_point(&self, resize_type: WResize) -> UVec2 {
        match resize_type {
            WResize::TopLeft => UVec2::new(self.rect.min.x, self.rect.min.y),
            WResize::TopRight => UVec2::new(self.rect.max.x, self.rect.min.y),
            WResize::BottomRight => UVec2::new(self.rect.max.x, self.rect.max.y),
            WResize::BottomLeft => UVec2::new(self.rect.min.x, self.rect.max.y),
            WResize::Radius => todo!(),
        }
    }

    fn contains(&self, x: u32, y: u32) -> bool {
        if self.is_hollow {
            return self.edge_contains(x, y);
        }
        x >= self.rect.min.x && x <= self.rect.max.x && y >= self.rect.min.y && y <= self.rect.max.y
    }

    fn edge_contains(&self, x: u32, y: u32) -> bool {
        ((x == self.rect.min.x || x == self.rect.max.x)
            && (y >= self.rect.min.y && y <= self.rect.max.y))
            || ((y == self.rect.min.y || y == self.rect.max.y)
                && (x >= self.rect.min.x && x <= self.rect.max.x))
    }

    fn is_deletable(&self) -> bool {
        self.rect.width() == 1 || self.rect.height() == 1
    }

    fn set_center(&mut self, x: u32, y: u32) {
        //TODO: out of bounds check
        let x = x as i32;
        let y = y as i32;

        let width = self.rect.width() as i32;
        let height = self.rect.height() as i32;
        let x0 = x - width / 2;
        let y0 = y - height / 2;
        let x1 = x0 + width - 1;
        let y1 = y0 + height - 1;

        if x0 < 0 {
            self.rect.min.x = 0;
            self.rect.max.x = width as u32 - 1;
        } else if x1 > SIMULATION_WIDTH as i32 {
            self.rect.max.x = SIMULATION_WIDTH - 1;
            self.rect.min.x = SIMULATION_WIDTH - width as u32;
        } else {
            self.rect.min.x = x0 as u32;
            self.rect.max.x = x1 as u32;
        }

        self.rect = WRect::new(x0 as u32, y0 as u32, x1 as u32, y1 as u32);
    }

    fn get_reflection_factor(&self) -> f32 {
        self.reflection_factor
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
        id: usize,
    ) -> Self {
        RectWall {
            rect: WRect::new(x0, y0, x1, y1),
            is_hollow,
            reflection_factor,
            id,
        }
    }

    pub fn set_top_left(&mut self, x: u32, y: u32) {
        
    }

    pub fn set_top_right(&mut self, x: u32, y: u32) {
        
    }

    pub fn set_bottom_left(&mut self, x: u32, y: u32) {
        
    }

    pub fn set_bottom_right(&mut self, mut x: u32, mut y: u32) {
        // make sure x and y are never less than min
        if x < self.rect.min.x {
            x = self.rect.min.x;
        }
        if y < self.rect.min.y {
            y = self.rect.min.y;
        }

        self.rect.max = UVec2::new(x, y);
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
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

    fn get_reflection_factor(&self) -> f32 {
        self.reflection_factor
    }
}

impl CircWall {
    pub fn new(
        x: u32,
        y: u32,
        radius: u32,
        is_hollow: bool,
        reflection_factor: f32,
        id: usize,
    ) -> Self {
        todo!()
    }

    pub fn set_radius() {}
}
