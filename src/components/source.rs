use std::f32::consts::PI;

use bevy::prelude::*;

use crate::math::constants::*;

#[derive(Component)]
pub struct Drag;

#[derive(Debug, Default, Component)]
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
}

#[derive(Debug, Default, PartialEq)]
pub enum SourceType {
    #[default]
    Sin,
    Gauss,
}

impl Source {
    pub fn new(
        x: u32,
        y: u32,
        amplitude: f32,
        phase: f32,
        frequency: f32,
        source_type: SourceType,
    ) -> Self {
        Self {
            x,
            y,
            phase,
            frequency,
            amplitude,
            source_type,
        }
    }

    pub fn calc(&self, time: f32) -> f32 {
        match self.source_type {
            SourceType::Sin => self.sin(time),
            SourceType::Gauss => Source::periodic_gaussian(1., 1., 1., 1., 1.),
        }
    }

    fn sin(&self, time: f32) -> f32 {
        self.amplitude * (2. * PI * self.frequency * (time - self.phase * PI / 180.)).sin()
    }

    /// generated by chat gpt, not sure if it's correct
    fn periodic_gaussian(x: f32, period: f32, amplitude: f32, mean: f32, variance: f32) -> f32 {
        // Ensure x is within the periodic domain [0, period)
        let x = (x % period + period) % period;

        // Calculate the Gaussian function value
        let exp_term = (-0.5 * ((x - mean) / variance).powi(2)).exp();
        let scaling_factor = 1.0 / (variance * (2.0 * PI).sqrt());

        amplitude * scaling_factor * exp_term
    }

    pub fn spawn_initial_sources(mut commands: Commands) {
        commands.spawn(Source::new(
            (SIMULATION_WIDTH + 2 * E_AL) / 2,
            (SIMULATION_HEIGHT + 2 * E_AL) / 2,
            10.,
            0.0,
            10000.0,
            SourceType::Sin,
        ));
        commands.spawn(Source::new(
            (SIMULATION_WIDTH + 2 * E_AL) / 3,
            (SIMULATION_HEIGHT + 2 * E_AL) / 3,
            10.,
            0.0,
            10000.0,
            SourceType::Sin,
        ));
    }
}
