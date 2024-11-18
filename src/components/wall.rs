use std::cmp::Ordering;
use std::f32::consts::TAU;

use bevy::prelude::*;
use egui::epaint::{CircleShape, TextShape};
use egui::text::LayoutJob;
use egui::{Align2, Color32, Pos2, Rect, TextFormat};
use serde::{Deserialize, Serialize};

use super::gizmo::GizmoComponent;
use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::rect::WRect;
use crate::math::transformations::grid_to_image;
use crate::render::gradient::Gradient;
use crate::ui::state::{PlaceType, ToolType};

#[derive(Debug, Default, Clone)]
pub struct WallCell {
    pub is_wall: bool,
    pub reflection_factor: f32,
    pub draw_reflection_factor: f32,
}

#[derive(Component, PartialEq)]
pub enum WResize {
    Menu,
    Draw,
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
    fn contains(&self, x: u32, y: u32) -> bool;

    fn edge_contains(&self, x: u32, y: u32) -> bool;

    fn boundary_delete(&self, x: u32, y: u32, boundary_width: u32) -> bool;

    /// If width or height equals one, the wall can be deleted
    fn is_deletable(&self) -> bool;

    fn set_center(&mut self, x: u32, y: u32);

    fn get_center(&self) -> UVec2;

    fn get_reflection_factor(&self) -> f32;

    fn get_resize_point(&self, resize_type: &WResize) -> UVec2;

    fn resize(&mut self, resize_type: &WResize, x: u32, y: u32);

    fn contains_pointer(&self, x: u32, y: u32) -> bool;
}

#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub struct RectWall {
    // between 0 and SIM_WIDTH
    // between 0 and SIM_HEIGHT
    pub rect: WRect,
    pub is_hollow: bool,
    pub reflection_factor: f32,
    pub id: usize,
    draw_pin: UVec2,
}

impl Wall for RectWall {
    fn get_center(&self) -> UVec2 {
        self.rect.center()
    }

    fn get_resize_point(&self, resize_type: &WResize) -> UVec2 {
        debug_assert!(
            resize_type != &WResize::Radius,
            "RectWall cannot be resized with WResize::Radius"
        );
        match resize_type {
            WResize::Draw => self.rect.max,
            WResize::TopLeft => self.rect.min,
            WResize::TopRight => UVec2::new(self.rect.max.x, self.rect.min.y),
            WResize::BottomRight => self.rect.max,
            WResize::BottomLeft => UVec2::new(self.rect.min.x, self.rect.max.y),
            WResize::Radius => unreachable!(),
            WResize::Menu => self.rect.center(),
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
            resize_type != &WResize::Radius,
            "RectWall cannot be resized with WResize::Radius"
        );
        match resize_type {
            WResize::TopLeft => {
                if x > self.rect.max.x - 1 {
                    x = self.rect.max.x - 1;
                }
                if y > self.rect.max.y - 1 {
                    y = self.rect.max.y - 1;
                }

                self.rect.min.x = x;
                self.rect.min.y = y;
            }
            WResize::TopRight => {
                if x < self.rect.min.x + 1 {
                    x = self.rect.min.x + 1;
                }
                if y > self.rect.max.y - 1 {
                    y = self.rect.max.y - 1;
                }

                self.rect.max.x = x;
                self.rect.min.y = y;
            }
            WResize::BottomRight => {
                // make sure x and y are never less than min
                // wall is always 2 pixel tall and wide
                if x < self.rect.min.x + 1 {
                    x = self.rect.min.x + 1;
                }
                if y < self.rect.min.y + 1 {
                    y = self.rect.min.y + 1;
                }

                self.rect.max.x = x;
                self.rect.max.y = y;
            }
            WResize::BottomLeft => {
                if x > self.rect.max.x - 1 {
                    x = self.rect.max.x - 1;
                }
                if y < self.rect.min.y + 1 {
                    y = self.rect.min.y + 1;
                }

                self.rect.min.x = x;
                self.rect.max.y = y;
            }
            WResize::Draw => {
                if x >= self.draw_pin.x {
                    self.rect.min.x = self.draw_pin.x;
                    self.rect.max.x = x;
                } else {
                    self.rect.min.x = x;
                    self.rect.max.x = self.draw_pin.x;
                }

                if y >= self.draw_pin.y {
                    self.rect.min.y = self.draw_pin.y;
                    self.rect.max.y = y;
                } else {
                    self.rect.min.y = y;
                    self.rect.max.y = self.draw_pin.y;
                }
            }
            WResize::Menu => {}
            WResize::Radius => unreachable!(),
        }
    }

    // x and y: 0..SIM_WIDTH/HEIGHT + 2 * B_W
    fn boundary_delete(&self, x: u32, y: u32, boundary_width: u32) -> bool {
        if self.rect.min.x == 0
            && x < self.rect.min.x + boundary_width
            && y >= self.rect.min.y + boundary_width
            && y <= self.rect.max.y + boundary_width
        {
            return true;
        }
        if self.rect.max.x == SIMULATION_WIDTH - 1
            && x > self.rect.max.x + boundary_width
            && y >= self.rect.min.y + boundary_width
            && y <= self.rect.max.y + boundary_width
        {
            return true;
        }

        if self.rect.min.y == 0
            && y < self.rect.min.y + boundary_width
            && x >= self.rect.min.x + boundary_width
            && x <= self.rect.max.x + boundary_width
        {
            return true;
        }
        if self.rect.max.y == SIMULATION_HEIGHT - 1
            && y > self.rect.max.y + boundary_width
            && x >= self.rect.min.x + boundary_width
            && x <= self.rect.max.x + boundary_width
        {
            return true;
        }
        false
    }

    fn contains_pointer(&self, x: u32, y: u32) -> bool {
        self.rect.min.x <= x && x <= self.rect.max.x && self.rect.min.y <= y && y <= self.rect.max.y
    }
}

impl GizmoComponent for RectWall {
    fn get_gizmo_positions(&self, tool_type: &ToolType) -> Vec<Pos2> {
        match tool_type {
            ToolType::ResizeWall => {
                let top_left = Pos2 {
                    x: self.rect.min.x as f32,
                    y: self.rect.min.y as f32,
                };
                let top_right = Pos2 {
                    x: self.rect.max.x as f32,
                    y: self.rect.min.y as f32,
                };
                let bottom_left = Pos2 {
                    x: self.rect.min.x as f32,
                    y: self.rect.max.y as f32,
                };
                let bottom_right = Pos2 {
                    x: self.rect.max.x as f32,
                    y: self.rect.max.y as f32,
                };

                vec![top_left, top_right, bottom_left, bottom_right]
            }
            ToolType::Move | ToolType::Select => {
                let center = self.get_center();
                vec![Pos2 {
                    x: center.x as f32,
                    y: center.y as f32,
                }]
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn draw_gizmo(
        &self,
        painter: &egui::Painter,
        tool_type: &ToolType,
        highlight: bool,
        image_rect: &Rect,
        _text: Option<&str>,
        delta_l: f32,
        current_gradient: Gradient,
    ) {
        let gizmo_color = match current_gradient {
            Gradient::Turbo => Color32::from_rgb(1, 89, 88),
            _ => Color32::from_rgb(15, 194, 192),
        };

        match tool_type {
            ToolType::ResizeWall => {
                for pos in self.get_gizmo_positions(tool_type) {
                    painter.add(egui::Shape::Circle(CircleShape::filled(
                        grid_to_image(pos, image_rect),
                        if highlight { 10. } else { 5. },
                        gizmo_color,
                    )));
                }

                self.draw_scale_text(painter, image_rect, delta_l, Color32::WHITE);
            }
            ToolType::Move | ToolType::Select => {
                for pos in self.get_gizmo_positions(tool_type) {
                    painter.add(egui::Shape::Circle(CircleShape::filled(
                        grid_to_image(pos, image_rect),
                        if highlight { 10. } else { 5. },
                        gizmo_color,
                    )));
                }

                self.draw_scale_text(painter, image_rect, delta_l, Color32::WHITE);
            }
            ToolType::Place(PlaceType::RectWall) => {
                self.draw_scale_text(painter, image_rect, delta_l, Color32::WHITE);
            }
            _ => {}
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
            draw_pin: UVec2 { x: x0, y: y0 },
        }
    }

    fn draw_scale_text(
        &self,
        painter: &egui::Painter,
        image_rect: &Rect,
        delta_l: f32,
        text_color: Color32,
    ) {
        let galley = {
            let layout_job = LayoutJob::single_section(
                format!("{:.3} m", self.rect.width() as f32 * delta_l),
                TextFormat {
                    color: text_color,
                    background: Color32::BLACK.gamma_multiply(0.75),
                    ..Default::default()
                },
            );
            painter.layout_job(layout_job)
        };
        let rect = Align2::CENTER_TOP.anchor_size(
            grid_to_image(
                Pos2 {
                    x: self.get_center().x as f32,
                    // +2. for some padding
                    y: self.rect.min.y as f32 + 6.,
                },
                image_rect,
            ),
            galley.size(),
        );
        painter.add(TextShape::new(rect.min, galley, Color32::BLACK));

        let galley = {
            let layout_job = LayoutJob::single_section(
                format!("{:.3} m", self.rect.height() as f32 * delta_l),
                TextFormat {
                    color: text_color,
                    background: Color32::BLACK.gamma_multiply(0.75),
                    ..Default::default()
                },
            );
            painter.layout_job(layout_job)
        };
        let rect = Align2::LEFT_CENTER.anchor_size(
            grid_to_image(
                Pos2 {
                    x: self.rect.min.x as f32 + 4.,
                    y: self.get_center().y as f32,
                },
                image_rect,
            ),
            galley.size(),
        );
        painter.add(
            TextShape::new(
                Pos2 {
                    x: rect.min.x + 2.,
                    y: rect.center().y as f32 + rect.width() / 2.,
                },
                galley,
                Color32::BLACK,
            )
            .with_angle(-std::f32::consts::FRAC_PI_2),
        );
    }
}

#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub struct CircWall {
    pub center: UVec2,
    /// Radius excludes center point
    /// r = 1 creates a three pixel wide/tall circle
    pub radius: u32,
    pub is_hollow: bool,
    pub reflection_factor: f32,
    //TODO: Better description
    /// open segment from x-axis (mirrored) in degrees
    pub open_circ_segment: f32,
    /// rotation angle in degrees
    pub rotation_angle: f32,
    pub id: usize,
    resize_point: UVec2,
}

impl Wall for CircWall {
    fn get_center(&self) -> UVec2 {
        self.center
    }

    fn get_resize_point(&self, resize_type: &WResize) -> UVec2 {
        match resize_type {
            WResize::Radius => UVec2 {
                x: self.resize_point.x,
                y: self.resize_point.y,
            },
            _ => {
                unreachable!()
            }
        }
    }

    fn contains(&self, x: u32, y: u32) -> bool {
        if self.is_hollow {
            return false;
        }
        let r_squared = self.radius * self.radius;

        (self.center.x as i32 - x as i32).pow(2) + (self.center.y as i32 - y as i32).pow(2)
            < r_squared as i32
    }

    fn edge_contains(&self, _x: u32, _y: u32) -> bool {
        panic!("use bresenham's algorithm to draw circular walls")
    }

    fn is_deletable(&self) -> bool {
        self.radius == 0
    }

    fn set_center(&mut self, x: u32, y: u32) {
        self.center.x = x;
        self.center.y = y;
    }

    fn get_reflection_factor(&self) -> f32 {
        self.reflection_factor
    }

    fn resize(&mut self, resize_type: &WResize, x: u32, y: u32) {
        match resize_type {
            WResize::Radius => {
                self.resize_point.x = x;
                self.resize_point.y = y;

                // angle in [0, 2pi)
                let angle = if (y as i32 - self.center.y as i32) <= 0 {
                    ((x as f32 - self.center.x as f32) / self.radius as f32).acos()
                //exp before acos can be > 1.0 (prob pixel inaccuracies)
                } else {
                    TAU - ((x as f32 - self.center.x as f32) / self.radius as f32).acos()
                };

                if !angle.is_nan() {
                    self.rotation_angle = (TAU - angle).to_degrees();
                }

                let x_offset = self.center.x as i32 - x as i32;
                let y_offset = self.center.y as i32 - y as i32;
                self.radius = ((x_offset.pow(2) + y_offset.pow(2)) as f32).sqrt() as u32;
            }
            WResize::Menu => {}
            _ => {
                panic!("Circular walls can only be resized by radius.");
            }
        }
    }

    fn boundary_delete(&self, x: u32, y: u32, boundary_width: u32) -> bool {
        let b_center_x = self.center.x + boundary_width;
        let b_center_y = self.center.y + boundary_width;

        if (x < boundary_width
            && y == b_center_y
            && (b_center_x as i32 - self.radius as i32) <= boundary_width as i32)
            || (x >= SIMULATION_WIDTH + boundary_width
                && y == b_center_y
                && b_center_x + self.radius >= SIMULATION_WIDTH + boundary_width)
        {
            return true;
        }

        if (y < boundary_width
            && x == b_center_x
            && (b_center_y as i32 - self.radius as i32) <= boundary_width as i32)
            || (y >= SIMULATION_HEIGHT + boundary_width
                && x == b_center_x
                && b_center_y + self.radius >= SIMULATION_HEIGHT + boundary_width)
        {
            return true;
        }
        false
    }

    fn contains_pointer(&self, x: u32, y: u32) -> bool {
        let x = x as i32;
        let y = y as i32;
        let center = self.get_center();
        let cx = center.x as i32;
        let cy = center.y as i32;

        (x - cx) * (x - cx) + (y - cy) * (y - cy) <= self.radius as i32 * self.radius as i32
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
        CircWall {
            center: UVec2 { x, y },
            radius,
            is_hollow,
            reflection_factor,
            open_circ_segment: 0.,
            rotation_angle: 0.,
            id,
            resize_point: UVec2 { x, y },
        }
    }

    fn draw_scale_text(
        &self,
        painter: &egui::Painter,
        image_rect: &Rect,
        delta_l: f32,
        text_color: Color32,
    ) {
        let galley = {
            let layout_job = LayoutJob::single_section(
                format!("\u{00D8} = {:.3} m", 2. * self.radius as f32 * delta_l),
                TextFormat {
                    color: text_color,
                    background: Color32::BLACK.gamma_multiply(0.75),
                    ..Default::default()
                },
            );
            painter.layout_job(layout_job)
        };
        let rect = Align2::CENTER_CENTER.anchor_size(
            grid_to_image(
                Pos2 {
                    x: self.get_center().x as f32,
                    y: self.get_center().y as f32,
                },
                image_rect,
            ),
            galley.size(),
        );
        painter.add(TextShape::new(rect.min, galley, Color32::BLACK));
    }
}

impl GizmoComponent for CircWall {
    fn get_gizmo_positions(&self, tool_type: &ToolType) -> Vec<Pos2> {
        match tool_type {
            ToolType::ResizeWall => {
                let resize_point = self.get_resize_point(&WResize::Radius);
                vec![Pos2 {
                    x: resize_point.x as f32,
                    y: resize_point.y as f32,
                }]
            }
            ToolType::Move | ToolType::Select => {
                vec![Pos2 {
                    x: self.center.x as f32,
                    y: self.center.y as f32,
                }]
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn draw_gizmo(
        &self,
        painter: &egui::Painter,
        tool_type: &ToolType,
        highlight: bool,
        image_rect: &Rect,
        _text: Option<&str>,
        delta_l: f32,
        current_gradient: Gradient,
    ) {
        let gizmo_color = match current_gradient {
            Gradient::Turbo => Color32::from_rgb(1, 89, 88),
            _ => Color32::from_rgb(15, 194, 192),
        };

        match tool_type {
            ToolType::ResizeWall => {
                for pos in self.get_gizmo_positions(tool_type) {
                    painter.add(egui::Shape::Circle(CircleShape::filled(
                        grid_to_image(pos, image_rect),
                        if highlight { 10. } else { 5. },
                        gizmo_color,
                    )));
                }
                self.draw_scale_text(painter, image_rect, delta_l, Color32::WHITE);
            }
            ToolType::Move | ToolType::Select => {
                for pos in self.get_gizmo_positions(tool_type) {
                    painter.add(egui::Shape::Circle(CircleShape::filled(
                        grid_to_image(pos, image_rect),
                        if highlight { 10. } else { 5. },
                        gizmo_color,
                    )));
                }
            }
            ToolType::Place(PlaceType::CircWall) => {
                self.draw_scale_text(painter, image_rect, delta_l, Color32::WHITE);
            }
            _ => {}
        }
    }
}
