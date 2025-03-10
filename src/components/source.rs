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
use crate::render::gradient::Gradient;
use crate::simulation::plugin::{ComponentIDs, WaveSamples};
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
    PeriodicGauss {
        /// phase shift of the bell (in °)
        phase: f32,
        /// frequency of the bell (in Hz)
        frequency: f32,
        /// amplitude of the bell (currently unitless)
        amplitude: f32,
        std_dev: f32,
    },
    GaussImpulse {
        /// amplitude of the bell (currently unitless)
        amplitude: f32,
        std_dev: f32,
    },
    WhiteNoise {
        /// amplitude of the noise (currently unitless)
        amplitude: f32,
    },
    WaveFile {
        amplitude: f32,
    },
}

impl Default for SourceType {
    fn default() -> Self {
        SourceType::Sin {
            amplitude: 10.,
            phase: 0.0,
            frequency: 1000.0,
        }
    }
}

impl SourceType {
    pub fn default_sin() -> SourceType {
        SourceType::Sin {
            amplitude: 10.,
            phase: 0.0,
            frequency: 1000.0,
        }
    }
    pub fn default_periodic_gauss() -> SourceType {
        SourceType::PeriodicGauss {
            amplitude: 10.,
            phase: 0.0,
            frequency: 1000.0,
            std_dev: 0.45,
        }
    }
    pub fn default_gauss_impulse() -> SourceType {
        SourceType::GaussImpulse {
            amplitude: 1.,
            std_dev: 0.001,
        }
    }
    pub fn default_noise() -> SourceType {
        SourceType::WhiteNoise { amplitude: 10. }
    }
    pub fn default_wave() -> SourceType {
        SourceType::WaveFile { amplitude: 100. } // bro wtf why 100
    }
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Sin { .. } => write!(f, "Sinusoidal"),
            SourceType::PeriodicGauss { .. } => write!(f, "Periodic Gaussian"),
            SourceType::GaussImpulse { .. } => write!(f, "Gaussian Impulse"),
            SourceType::WhiteNoise { .. } => write!(f, "White noise"),
            SourceType::WaveFile { .. } => write!(f, "Wave file"),
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

    pub fn calc(&self, time: f32, cur_sample: usize, wave_samples: &WaveSamples) -> f32 {
        match self.source_type {
            SourceType::Sin {
                phase,
                frequency,
                amplitude,
            } => self.sin(time, phase, frequency, amplitude),
            SourceType::PeriodicGauss {
                phase,
                amplitude,
                frequency,
                std_dev,
            } => self.periodic_gaussian(time, frequency, amplitude, phase, 4., 0., std_dev),
            SourceType::GaussImpulse { amplitude, std_dev } => {
                self.gaussian_impulse(time, amplitude, 0.001, std_dev)
            }
            SourceType::WhiteNoise { amplitude } => {
                thread_rng().sample::<f32, _>(rand_distr::StandardNormal) * amplitude
            }
            SourceType::WaveFile { amplitude } => {
                wave_samples.0[cur_sample % wave_samples.0.len()] * amplitude
            }
        }
    }

    fn sin(&self, time: f32, phase: f32, frequency: f32, amplitude: f32) -> f32 {
        if time < phase / (frequency * 360.) {
            return 0.;
        }
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

    fn gaussian_impulse(
        &self,
        time: f32,
        amplitude: f32,
        mean: f32,
        standard_deviation: f32,
    ) -> f32 {
        let exp_term = (-0.5 * ((2. * PI * time - mean) / standard_deviation).powi(2)).exp();
        let scaling_factor = 1.0 / (standard_deviation * (2.0 * PI).sqrt());

        amplitude * scaling_factor * exp_term
    }

    pub fn spawn_initial_sources(mut commands: Commands, mut component_ids: ResMut<ComponentIDs>) {
        commands.spawn(Source::new(
            (SIMULATION_WIDTH + 2 * INIT_BOUNDARY_WIDTH) / 2,
            (SIMULATION_HEIGHT + 2 * INIT_BOUNDARY_WIDTH) / 2,
            SourceType::default_sin(),
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
    fn get_gizmo_positions(&self, _tool_type: &ToolType) -> Vec<Pos2> {
        vec![Pos2 {
            x: self.x as f32,
            y: self.y as f32,
        }]
    }

    fn draw_gizmo(
        &self,
        painter: &egui::Painter,
        tool_type: &ToolType,
        highlight: bool,
        image_rect: &Rect,
        text: Option<&str>,
        _delta_l: f32,
        current_gradient: Gradient,
    ) {
        let (gizmo_color, text_color) = match current_gradient {
            Gradient::Turbo => (Color32::from_rgb(1, 89, 88), Color32::WHITE),
            _ => (Color32::from_rgb(15, 194, 192), Color32::BLACK),
        };

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
}
