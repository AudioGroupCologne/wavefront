use std::cmp::Ordering;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};

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
    Radius,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct WallPos2 {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct WallRect {
    pub min: WallPos2,
    pub max: WallPos2,
}

impl WallRect {
    pub fn new(min_x: u32, min_y: u32, max_x: u32, max_y: u32) -> Self {
        WallRect {
            min: WallPos2 { x: min_x, y: min_y },
            max: WallPos2 { x: max_x, y: max_y },
        }
    }
    pub fn center(&self) -> WallPos2 {
        WallPos2 {
            x: (self.min.x + self.max.x) / 2,
            y: (self.min.y + self.max.y) / 2,
        }
    }

    // these will make problems with the 'normal' rect
    // + 1 because walls are inclusive
    pub fn width(&self) -> u32 {
        self.max.x - self.min.x + 1
    }

    pub fn height(&self) -> u32 {
        self.max.y - self.min.y + 1
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum WallType {
    Rectangle,
    Circle,
}

#[derive(Debug, Component, Serialize, Deserialize, Clone)]
pub struct Wall {
    pub wall_type: WallType,
    pub hollow: bool,
    pub rect: WallRect,
    pub draw_rect: WallRect,
    pub calc_rect: WallRect,
    pub center: WallPos2,
    pub reflection_factor: f32,
    pub id: usize,
}

impl Wall {
    pub fn new(
        wall_type: WallType,
        hollow: bool,
        rect: WallRect,
        reflection_factor: f32,
        id: usize,
    ) -> Self {
        Self {
            wall_type,
            hollow,
            rect,
            draw_rect: rect,
            calc_rect: rect,
            center: rect.center(),
            reflection_factor,
            id,
        }
    }

    pub fn get_center(&self) -> WallPos2 {
        self.draw_rect.center()
    }

    pub fn get_resize_point(&self) -> WallPos2 {
        self.draw_rect.max
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.calc_rect.min.x
            && x <= self.calc_rect.max.x
            && y >= self.calc_rect.min.y
            && y <= self.calc_rect.max.y
    }

    pub fn edge_contains(&self, x: u32, y: u32) -> bool {
        ((x == self.calc_rect.min.x || x == self.calc_rect.max.x)
            && (y >= self.calc_rect.min.y && y <= self.calc_rect.max.y))
            || ((y == self.calc_rect.min.y || y == self.calc_rect.max.y)
                && (x >= self.calc_rect.min.x && x <= self.calc_rect.max.x))
    }

    pub fn is_empty(&self) -> bool {
        self.draw_rect.width() == 1 || self.draw_rect.height() == 1
    }

    pub fn translate_center_to(&mut self, x: u32, y: u32, e_al: u32) {
        let current_center = self.draw_rect.center();

        let mut x_offset = x as i32 - current_center.x as i32;
        let mut y_offset = y as i32 - current_center.y as i32;

        match x_offset.cmp(&0) {
            Ordering::Less => {
                x_offset = if x_offset.abs() > self.draw_rect.min.x as i32 {
                    self.draw_rect.min.x as i32
                } else {
                    x_offset
                };
                self.rect.min.x -= x_offset.unsigned_abs();
                self.rect.max.x -= x_offset.unsigned_abs();
            }
            Ordering::Greater => {
                // minus 1 because wall-bounds are inclusive
                x_offset = if x_offset > SIMULATION_WIDTH as i32 - self.draw_rect.max.x as i32 - 1 {
                    SIMULATION_WIDTH as i32 - self.draw_rect.max.x as i32 - 1
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
                y_offset = if y_offset.abs() > self.draw_rect.min.y as i32 {
                    self.draw_rect.min.y as i32
                } else {
                    y_offset
                };
                self.rect.min.y -= y_offset.unsigned_abs();
                self.rect.max.y -= y_offset.unsigned_abs();
            }
            Ordering::Greater => {
                // minus 1 because wall-bounds are inclusive
                y_offset = if y_offset > SIMULATION_HEIGHT as i32 - self.draw_rect.max.y as i32 - 1
                {
                    SIMULATION_HEIGHT as i32 - self.draw_rect.max.y as i32 - 1
                } else {
                    y_offset
                };
                self.rect.min.y += y_offset as u32;
                self.rect.max.y += y_offset as u32;
            }
            _ => {}
        }

        self.update_calc_rect(e_al);
    }

    pub fn update_calc_rect(&mut self, e_al: u32) {
        // Q 4
        if self.rect.min.x <= self.rect.max.x && self.rect.min.y <= self.rect.max.y {
            self.draw_rect.min = self.rect.min;
            self.draw_rect.max = self.rect.max;
            // Q 3
        } else if self.rect.min.x > self.rect.max.x && self.rect.min.y < self.rect.max.y {
            self.draw_rect.min.x = self.rect.max.x;
            self.draw_rect.min.y = self.rect.min.y;
            self.draw_rect.max.x = self.rect.min.x;
            self.draw_rect.max.y = self.rect.max.y;
            // Q 2
        } else if self.rect.min.x > self.rect.max.x && self.rect.min.y > self.rect.max.y {
            self.draw_rect.min = self.rect.max;
            self.draw_rect.max = self.rect.min;
            // Q1
        } else if self.rect.min.x < self.rect.max.x && self.rect.min.y > self.rect.max.y {
            self.draw_rect.min.x = self.rect.min.x;
            self.draw_rect.min.y = self.rect.max.y;
            self.draw_rect.max.x = self.rect.max.x;
            self.draw_rect.max.y = self.rect.min.y;
        }

        // For Hollow Walls this doesn't work
        // self.calc_rect.min.x = if self.draw_rect.min.x == 0 {
        //     // there is no check for the very corner if we can access the array,
        //     // so this is the lazy version
        //     1
        // } else {
        //     self.draw_rect.min.x + e_al
        // };
        // self.calc_rect.min.y = if self.draw_rect.min.y == 0 {
        //     1
        // } else {
        //     self.draw_rect.min.y + e_al
        // };
        // self.calc_rect.max.x = if self.draw_rect.max.x == SIMULATION_WIDTH - 1 {
        //     SIMULATION_WIDTH + 2 * e_al - 2
        // } else {
        //     self.draw_rect.max.x + e_al
        // };
        // self.calc_rect.max.y = if self.draw_rect.max.y == SIMULATION_HEIGHT - 1 {
        //     SIMULATION_HEIGHT + 2 * e_al - 2
        // } else {
        //     self.draw_rect.max.y + e_al
        // };

        self.calc_rect.min.x = self.draw_rect.min.x + e_al;
        self.calc_rect.min.y = self.draw_rect.min.y + e_al;
        self.calc_rect.max.x = self.draw_rect.max.x + e_al;
        self.calc_rect.max.y = self.draw_rect.max.y + e_al;
    }
}
