use bevy::prelude::*;
use egui::epaint::{CircleShape, TextShape};
use egui::text::LayoutJob;
use egui::{Align2, Color32, Pos2, Rect, TextFormat};
use serde::{Deserialize, Serialize};

use super::gizmo::GizmoComponent;
use crate::math::transformations::grid_to_image;
use crate::render::gradient::Gradient;
use crate::simulation::plugin::ComponentIDs;
use crate::ui::state::ToolType;

/// A microphone on the grid that records the pressure at its position
#[derive(Debug, Default, Component, Serialize, Deserialize, Clone, PartialEq)]
pub struct Microphone {
    pub x: u32,
    pub y: u32,
    pub id: usize,
    #[serde(skip_serializing, skip_deserializing)]
    pub record: Vec<[f64; 2]>,
    pub show_fft: bool,
}

impl Microphone {
    pub fn new(x: u32, y: u32, id: usize) -> Self {
        Self {
            x,
            y,
            id,
            record: vec![],
            show_fft: false,
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

    pub fn write_to_file(&mut self, path: &str) {
        let mut wtr = csv::Writer::from_path(path).unwrap();
        for record in &self.record {
            wtr.write_record(&[record[0].to_string(), record[1].to_string()])
                .unwrap();
        }
        wtr.flush().unwrap();
    }
}

impl GizmoComponent for Microphone {
    fn get_gizmo_positions(&self, tool_type: &ToolType) -> Vec<Pos2> {
        match tool_type {
            ToolType::Move | ToolType::Place(..) | ToolType::Select => {
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
        text: Option<&str>,
        _delta_l: f32,
        _current_gradient: Gradient,
    ) {
        let (gizmo_color, text_color) = (Color32::LIGHT_BLUE, Color32::BLACK);

        match tool_type {
            ToolType::Place(..) | ToolType::Move | ToolType::Select => {
                for pos in self.get_gizmo_positions(tool_type) {
                    painter.add(egui::Shape::Circle(CircleShape::filled(
                        grid_to_image(pos, image_rect),
                        if highlight { 15. } else { 10. },
                        gizmo_color,
                    )));
                    if let Some(text) = text {
                        let galley = {
                            let layout_job = LayoutJob::single_section(
                                text.to_owned(),
                                TextFormat {
                                    color: text_color,
                                    background: Color32::TRANSPARENT,
                                    ..Default::default()
                                },
                            );
                            painter.layout_job(layout_job)
                        };
                        let rect = Align2::CENTER_CENTER.anchor_size(
                            grid_to_image(
                                Pos2 {
                                    x: self.x as f32,
                                    y: self.y as f32,
                                },
                                image_rect,
                            ),
                            galley.size(),
                        );
                        painter.add(TextShape::new(rect.min, galley, Color32::BLACK));
                    }
                }
            }
            _ => {}
        }
    }
}
