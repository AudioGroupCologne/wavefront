use std::f32::consts::PI;
use std::fmt;

use bevy::prelude::*;
use egui::epaint::{CircleShape, TextShape};
use egui::text::LayoutJob;
use egui::{Align2, Color32, Pos2, Rect, TextFormat};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use super::gizmo::GizmoComponent;
use crate::math::constants::*;
use crate::math::transformations::grid_to_image;
use crate::simulation::plugin::ComponentIDs;
use crate::ui::state::ToolType;

/// A sound source on the grid
#[derive(Debug, Default, Component, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub struct Source {
    pub x: u32,
    pub y: u32,
    /// type of the source
    pub source_type: SourceType,
    pub id: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum SourceType {
    Sin {
        /// phase shift of the sin (in °)
        phase: f32,
        /// frequency of the sin (in Hz)
        frequency: f32,
        /// amplitude of the sin (currently unitless)
        amplitude: f32,
    },
    Gauss {
        /// phase shift of the bell (in °)
        phase: f32,
        /// frequency of the bell (in Hz)
        frequency: f32,
        /// amplitude of the bell (currently unitless)
        amplitude: f32,
        std_dev: f32,
    },
    WhiteNoise {
        /// amplitude of the noise (currently unitless)
        amplitude: f32,
    },
}

impl Default for SourceType {
    fn default() -> Self {
        SourceType::Sin {
            amplitude: 10.,
            phase: 0.0,
            frequency: 10000.0,
        }
    }
}

impl SourceType {
    pub fn default_sin() -> SourceType {
        SourceType::Sin {
            amplitude: 10.,
            phase: 0.0,
            frequency: 10000.0,
        }
    }
    pub fn default_gauss() -> SourceType {
        SourceType::Gauss {
            amplitude: 10.,
            phase: 0.0,
            frequency: 10000.0,
            std_dev: 0.45,
        }
    }
    pub fn default_noise() -> SourceType {
        SourceType::WhiteNoise { amplitude: 10. }
    }
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Sin { .. } => write!(f, "Sinusoidal"),
            SourceType::Gauss { .. } => write!(f, "Gaussian"),
            SourceType::WhiteNoise { .. } => write!(f, "White Noise"),
        }
    }
}

impl Source {
    pub fn new(x: u32, y: u32, source_type: SourceType, id: usize) -> Self {
        Self {
            x,
            y,
            source_type,
            id,
        }
    }

    pub fn calc(&self, time: f32) -> f32 {
        match self.source_type {
            SourceType::Sin {
                phase,
                frequency,
                amplitude,
            } => self.sin(time, phase, frequency, amplitude),
            SourceType::Gauss {
                phase,
                amplitude,
                frequency,
                std_dev
            } => self.periodic_gaussian(time, frequency, amplitude, phase, 4., 0., std_dev),
            SourceType::WhiteNoise { amplitude } => {
                thread_rng().sample::<f32, _>(rand_distr::StandardNormal) * amplitude
            }
        }
    }

    fn sin(&self, time: f32, phase: f32, frequency: f32, amplitude: f32) -> f32 {
        amplitude * (2. * PI * frequency * time - phase.to_radians()).sin()
    }

    fn periodic_gaussian(
        &self,
        time: f32,
        frequency: f32,
        amplitude: f32,
        phase: f32,
        period: f32,
        mean: f32,
        standard_deviation: f32,
    ) -> f32 {
        // Ensure x is within the periodic domain (-period/2 ; period/2)
        let x = ((2. * PI * frequency * time - phase.to_radians()) % period) - (period / 2.);

        // Calculate the Gaussian function value
        let exp_term = (-0.5 * ((x - mean) / standard_deviation).powi(2)).exp();
        let scaling_factor = 1.0 / (standard_deviation * (2.0 * PI).sqrt());

        amplitude * scaling_factor * exp_term
    }

    pub fn spawn_initial_sources(mut commands: Commands, mut component_ids: ResMut<ComponentIDs>) {
        commands.spawn(Source::new(
            (SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH) / 2,
            (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH) / 2,
            SourceType::Sin {
                amplitude: 10.,
                phase: 0.0,
                frequency: 10000.0,
            },
            component_ids.get_new_source_id(),
        ));
        commands.spawn(Source::new(
            (SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH) / 3,
            (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH) / 3,
            SourceType::default_sin(),
            component_ids.get_new_source_id(),
        ));
    }
}

impl GizmoComponent for Source {
    fn get_gizmo_positions(&self, tool_type: &ToolType) -> Vec<Pos2> {
        match tool_type {
            ToolType::Place(..) | ToolType::Move | ToolType::Select => {
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
    ) {
        match tool_type {
            ToolType::Place(..) | ToolType::Move | ToolType::Select => {
                for pos in self.get_gizmo_positions(tool_type) {
                    painter.add(egui::Shape::Circle(CircleShape::filled(
                        grid_to_image(pos, image_rect),
                        if highlight { 15. } else { 10. },
                        Color32::RED,
                    )));
                    if let Some(text) = text {
                        let galley = {
                            let layout_job = LayoutJob::single_section(
                                text.to_owned(),
                                TextFormat {
                                    color: Color32::WHITE,
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
