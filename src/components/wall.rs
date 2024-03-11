use std::cmp::Ordering;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::rect::WRect;

#[derive(Component, PartialEq)]
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

    fn resize(&mut self, resize_type: &WResize, x: u32, y: u32);
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
        debug_assert!(
            resize_type != WResize::Radius,
            "RectWall cannot be resized with WResize::Radius"
        );
        match resize_type {
            WResize::TopLeft => self.rect.min,
            WResize::TopRight => UVec2::new(self.rect.max.x, self.rect.min.y),
            WResize::BottomRight => self.rect.max,
            WResize::BottomLeft => UVec2::new(self.rect.min.x, self.rect.max.y),
            WResize::Radius => unreachable!(),
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
        let current_center = self.rect.center();

        let mut x_offset = x as i32 - current_center.x as i32;
        let mut y_offset = y as i32 - current_center.y as i32;

        match x_offset.cmp(&0) {
            Ordering::Less => {
                x_offset = if x_offset.abs() > self.rect.min.x as i32 {
                    self.rect.min.x as i32
                } else {
                    x_offset
                };
                self.rect.min.x -= x_offset.unsigned_abs();
                self.rect.max.x -= x_offset.unsigned_abs();
            }
            Ordering::Greater => {
                // minus 1 because wall-bounds are inclusive
                x_offset = if x_offset > SIMULATION_WIDTH as i32 - self.rect.max.x as i32 - 1 {
                    SIMULATION_WIDTH as i32 - self.rect.max.x as i32 - 1
                } else {
                    x_offset
                };
                self.rect.min.x += x_offset as u32;
                self.rect.max.x += x_offset as u32;
            }
            _ => {}
        }

        match y_offset.cmp(&0) {
            Ordering::Less => {
                y_offset = if y_offset.abs() > self.rect.min.y as i32 {
                    self.rect.min.y as i32
                } else {
                    y_offset
                };
                self.rect.min.y -= y_offset.unsigned_abs();
                self.rect.max.y -= y_offset.unsigned_abs();
            }
            Ordering::Greater => {
                // minus 1 because wall-bounds are inclusive
                y_offset = if y_offset > SIMULATION_HEIGHT as i32 - self.rect.max.y as i32 - 1 {
                    SIMULATION_HEIGHT as i32 - self.rect.max.y as i32 - 1
                } else {
                    y_offset
                };
                self.rect.min.y += y_offset as u32;
                self.rect.max.y += y_offset as u32;
            }
            _ => {}
        }
    }

    fn get_reflection_factor(&self) -> f32 {
        self.reflection_factor
    }

    fn resize(&mut self, resize_type: &WResize, mut x: u32, mut y: u32) {
        debug_assert!(
            *resize_type != WResize::Radius,
            "RectWall cannot be resized with WResize::Radius"
        );
        match resize_type {
            WResize::TopLeft => todo!(),
            WResize::TopRight => todo!(),
            WResize::BottomRight => {
                // make sure x and y are never less than min
                if x < self.rect.min.x {
                    x = self.rect.min.x;
                }
                if y < self.rect.min.y {
                    y = self.rect.min.y;
                }

                self.rect.max = UVec2::new(x, y);
            }
            WResize::BottomLeft => todo!(),
            WResize::Radius => unreachable!(),
        }
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

    fn resize(&mut self, resize_type: &WResize, x: u32, y: u32) {
        match resize_type {
            WResize::Radius => {}
            _ => {
                panic!("Circular walls cannot be resized by radius.");
            }
        }
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
}
