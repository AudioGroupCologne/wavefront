use bevy::prelude::*;
use egui::epaint::CircleShape;
use egui::{Color32, Pos2, Rect};
use serde::{Deserialize, Serialize};

use super::gizmo::GizmoComponent;
use crate::math::transformations::grid_to_image;
use crate::simulation::plugin::ComponentIDs;
use crate::ui::state::{PlaceType, ToolType};

/// A microphone on the grid that records the pressure at its position
#[derive(Debug, Default, Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct Microphone {
    pub x: u32,
    pub y: u32,
    pub id: usize,
    #[serde(skip_serializing, skip_deserializing)]
    pub record: Vec<[f64; 2]>,
}

impl Microphone {
    pub fn new(x: u32, y: u32, id: usize) -> Self {
        Self {
            x,
            y,
            id,
            record: vec![],
        }
    }

    pub fn spawn_initial_microphones(
        mut commands: Commands,
        mut component_ids: ResMut<ComponentIDs>,
    ) {
        commands.spawn(Microphone::new(250, 250, component_ids.get_new_mic_id()));
        commands.spawn(Microphone::new(100, 100, component_ids.get_new_mic_id()));
        commands.spawn(Microphone::new(650, 650, component_ids.get_new_mic_id()));
    }

    pub fn clear(&mut self) {
        self.record = vec![];
    }
}

impl GizmoComponent for Microphone {
    fn get_gizmo_positions(&self, tool_type: &ToolType) -> Vec<Pos2> {
        match tool_type {
            ToolType::Move | ToolType::Place(PlaceType::Mic) => {
                vec![Pos2 {
                    x: self.x as f32,
                    y: self.y as f32,
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
        _delta_l: f32,
        _text_color: Color32,
    ) {
        match tool_type {
            ToolType::Place(PlaceType::Mic) | ToolType::Move => {
                for pos in self.get_gizmo_positions(tool_type) {
                    painter.add(egui::Shape::Circle(CircleShape::filled(
                        grid_to_image(pos, image_rect),
                        if highlight { 10. } else { 5. },
                        Color32::GOLD,
                    )));
                }
            }
            _ => {}
        }
    }
}
