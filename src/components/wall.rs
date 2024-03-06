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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum WallType {
    Rectangle,
    Circle,
}

#[derive(Debug, Component, Serialize, Deserialize)]
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

    pub fn is_empty(&self) -> bool {
        self.draw_rect.width() == 1 || self.draw_rect.height() == 1
    }

    pub fn translate_center_to(&mut self, x: u32, y: u32, e_al: u32) {
        let current_center = self.draw_rect.center();

        let width = self.draw_rect.width();
        let height = self.draw_rect.height();

        let x_offset = x as i32 - current_center.x as i32;
        let y_offset = y as i32 - current_center.y as i32;

        let new_min_x = self.rect.min.x as i32 + x_offset;
        let new_min_y = self.rect.min.y as i32 + y_offset;
        let new_max_x = self.rect.max.x as i32 + x_offset;
        let new_max_y = self.rect.max.y as i32 + y_offset;

        if new_min_x >= 0 && new_max_x <= (SIMULATION_WIDTH - 1) as i32 {
            self.rect.min.x = new_min_x as u32;
            self.rect.max.x = new_max_x as u32;
        } else {
            if new_min_x < 0 {
                self.rect.min.x = 0;
                self.rect.max.x = width - 1;
            }
            if new_max_x > (SIMULATION_WIDTH - 1) as i32 {
                self.rect.min.x = SIMULATION_WIDTH - width;
                self.rect.max.x = SIMULATION_WIDTH - 1;
            }
        }
        if new_min_y >= 0 && new_max_y <= (SIMULATION_HEIGHT - 1) as i32 {
            self.rect.min.y = new_min_y as u32;
            self.rect.max.y = new_max_y as u32;
        } else {
            if new_min_y < 0 {
                self.rect.min.y = 0;
                self.rect.max.y = height - 1;
            }
            if new_max_y > (SIMULATION_HEIGHT - 1) as i32 {
                self.rect.min.y = SIMULATION_HEIGHT - height;
                self.rect.max.y = SIMULATION_HEIGHT - 1;
            }
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
