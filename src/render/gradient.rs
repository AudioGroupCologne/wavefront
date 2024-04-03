use bevy::ecs::system::Resource;
use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::math::transformations::map_range;

#[derive(Resource, Serialize, Deserialize, Clone, Copy)]
pub struct Gradient(pub Color32, pub Color32, pub Color32);

impl Gradient {
    pub fn new(start: Color32, middle: Color32, end: Color32) -> Self {
        Self(start, middle, end)
    }

    pub fn update(&mut self, gradient: &Gradient) {
        self.0 = gradient.0;
        self.1 = gradient.1;
        self.2 = gradient.2;
    }

    pub fn at(&self, percent: f32, contrast: f32) -> Color32 {
        let percent = percent * contrast;
        if percent >= 0. {
            self.upper(map_range(0., 5., 0., 1., percent))
        } else {
            self.lower(map_range(-5., 0., 0., 1., percent))
        }
    }

    fn lower(&self, percent: f32) -> Color32 {
        let result_red = self.0.r() as f32 + percent * (self.1.r() as f32 - self.0.r() as f32);
        let result_green = self.0.g() as f32 + percent * (self.1.g() as f32 - self.0.g() as f32);
        let result_blue = self.0.b() as f32 + percent * (self.1.b() as f32 - self.0.b() as f32);
        Color32::from_rgb(result_red as u8, result_green as u8, result_blue as u8)
    }

    fn upper(&self, percent: f32) -> Color32 {
        let result_red = self.1.r() as f32 + percent * (self.2.r() as f32 - self.1.r() as f32);
        let result_green = self.1.g() as f32 + percent * (self.2.g() as f32 - self.1.g() as f32);
        let result_blue = self.1.b() as f32 + percent * (self.2.b() as f32 - self.1.b() as f32);
        Color32::from_rgb(result_red as u8, result_green as u8, result_blue as u8)
    }

    //TODO: Rethink
    pub fn get_average(&self) -> Color32 {
        // let result_red = 255. - (self.0.r() as f32 + self.2.r() as f32) / 2.;
        // let result_green = (self.0.g() as f32 + self.2.g() as f32) / 2.;
        // let result_blue = (self.0.b() as f32 + self.2.b() as f32) / 2.;
        // Color32::from_rgb(result_red as u8, result_green as u8, result_blue as u8)
        Color32::WHITE
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Self(
            Color32::from_hex("#00f0c8").unwrap(),
            Color32::BLACK,
            Color32::WHITE,
        )
    }
}
