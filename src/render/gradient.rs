use bevy::ecs::system::Resource;
use egui::Color32;
use serde::{Deserialize, Serialize};

/// A gradient struct that holds three colors (negative, neutral, positive)
#[derive(Resource)]
pub struct Gradient(pub colorgrad::Gradient);

impl Gradient {
    /// Returns the color at a certain percentage (between 0.0..=1.0) of the gradient
    pub fn at(&self, percent: f32, _contrast: f32) -> Color32 {
        let [r, g, b, _] = self.0.at(percent as f64).to_rgba8();
        Color32::from_rgb(r, g, b)
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Gradient(colorgrad::turbo())
    }
}

#[derive(Resource, Clone, Copy, Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum GradientType {
    #[default]
    Turbo,
    Viridis,
    Magma,
    Plasma,
    Inferno,
    Cividis,
}

impl From<GradientType> for Gradient {
    fn from(gradient: GradientType) -> Self {
        match gradient {
            GradientType::Turbo => Gradient(colorgrad::turbo()),
            GradientType::Viridis => Gradient(colorgrad::viridis()),
            GradientType::Magma => Gradient(colorgrad::magma()),
            GradientType::Plasma => Gradient(colorgrad::plasma()),
            GradientType::Inferno => Gradient(colorgrad::inferno()),
            GradientType::Cividis => Gradient(colorgrad::cividis()),
        }
    }
}