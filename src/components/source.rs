use std::f32::consts::PI;
use std::fmt;

use bevy::prelude::*;
use egui::epaint::CircleShape;
use egui::{Color32, Pos2, Rect};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use super::gizmo::GizmoComponent;
use crate::grid::plugin::ComponentIDs;
use crate::math::constants::*;
use crate::math::transformations::grid_to_image;
use crate::ui::state::ToolType;

#[derive(Debug, Default, Component, Serialize, Deserialize, Clone, PartialEq, Copy)]
/// A sound source on the grid
pub struct Source {
    pub x: u32,
    pub y: u32,
    /// phase shift of the function in degrees
    pub phase: f32,
    /// frequency of the function (in Hz)
    pub frequency: f32,
    /// amplitude of the function (currently unitless)
    pub amplitude: f32,
    /// type of the source
    pub source_type: SourceType,
    pub id: usize,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum SourceType {
    #[default]
    Sin,
    Gauss,
    WhiteNoise,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Sin => write!(f, "Sinusoidal"),
            SourceType::Gauss => write!(f, "Gaussian"),
            SourceType::WhiteNoise => write!(f, "White Noise"),
        }
    }
}

impl Source {
    pub fn new(
        x: u32,
        y: u32,
        amplitude: f32,
        phase: f32,
        frequency: f32,
        source_type: SourceType,
        id: usize,
    ) -> Self {
        Self {
            x,
            y,
            phase,
            frequency,
            amplitude,
            source_type,
            id,
        }
    }

    pub fn calc(&self, time: f32) -> f32 {
        match self.source_type {
            SourceType::Sin => self.sin(time),
            SourceType::Gauss => self.periodic_gaussian(time, 4., 0., 0.45),
            SourceType::WhiteNoise => {
                thread_rng().sample::<f32, _>(rand_distr::StandardNormal) * self.amplitude
            }
        }
    }

    fn sin(&self, time: f32) -> f32 {
        self.amplitude * (2. * PI * self.frequency * (time - self.phase * PI / 180.)).sin()
    }

    fn periodic_gaussian(&self, time: f32, period: f32, mean: f32, standard_deviation: f32) -> f32 {
        // Ensure x is within the periodic domain (-period/2 ; period/2)
        let x = ((2. * PI * self.frequency * time) % period) - (period / 2.);

        // Calculate the Gaussian function value
        let exp_term = (-0.5 * ((x - mean) / standard_deviation).powi(2)).exp();
        let scaling_factor = 1.0 / (standard_deviation * (2.0 * PI).sqrt());

        self.amplitude * scaling_factor * exp_term
    }

    pub fn spawn_initial_sources(mut commands: Commands, mut component_ids: ResMut<ComponentIDs>) {
        commands.spawn(Source::new(
            (SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH) / 2,
            (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH) / 2,
            10.,
            0.0,
            10000.0,
            SourceType::Sin,
            component_ids.get_new_source_id(),
        ));
        commands.spawn(Source::new(
            (SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH) / 3,
            (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH) / 3,
            10.,
            0.0,
            10000.0,
            SourceType::Sin,
            component_ids.get_new_source_id(),
        ));
    }
}

impl GizmoComponent for Source {
    fn get_gizmo_positions(&self, tool_type: &ToolType) -> Vec<Pos2> {
        match tool_type {
            ToolType::PlaceSource | ToolType::MoveSource => {
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
    ) {
        match tool_type {
            ToolType::PlaceSource | ToolType::MoveSource => {
                for pos in self.get_gizmo_positions(tool_type) {
                    painter.add(egui::Shape::Circle(CircleShape::filled(
                        grid_to_image(pos, image_rect),
                        if highlight { 10. } else { 5. },
                        Color32::LIGHT_BLUE,
                    )));
                }
            }
            _ => {}
        }
    }
}
