use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::emath::Pos2;

use super::states::Gizmo;
use crate::grid::plugin::ComponentIDs;
use crate::math::constants::*;
use crate::math::transformations::f32_map_range;

#[derive(Debug, Default, Component)]
/// A microphone on the grid that records the pressure at its position
pub struct Microphone {
    pub x: u32,
    pub y: u32,
    pub id: usize,
    pub record: Vec<[f64; 2]>,
    pub spectrum: Vec<Vec<[f64; 2]>>,
}

impl Microphone {
    pub fn new(x: u32, y: u32, id: usize) -> Self {
        Self {
            x,
            y,
            id,
            record: vec![[0., 0.]],
            spectrum: vec![],
        }
    }

    pub fn spawn_initial_microphones(
        mut commands: Commands,
        mut component_ids: ResMut<ComponentIDs>,
    ) {
        commands.spawn(Microphone::new(
            250,
            250,
            component_ids.get_current_mic_id(),
        ));
        commands.spawn(Microphone::new(
            100,
            100,
            component_ids.get_current_mic_id(),
        ));
        commands.spawn(Microphone::new(
            650,
            650,
            component_ids.get_current_mic_id(),
        ));
    }

    pub fn clear(&mut self) {
        self.record = vec![[0., 0.]];
    }
}

impl Gizmo for Microphone {
    fn get_gizmo_position(&self, rect: &bevy_pixel_buffer::bevy_egui::egui::Rect) -> Pos2 {
        Pos2 {
            x: f32_map_range(
                0.,
                SIMULATION_WIDTH as f32,
                rect.min.x,
                rect.max.x,
                self.x as f32,
            ),
            y: f32_map_range(
                0.,
                SIMULATION_HEIGHT as f32,
                rect.min.y,
                rect.max.y,
                self.y as f32,
            ),
        }
    }

    fn get_gizmo_color(&self) -> bevy_pixel_buffer::bevy_egui::egui::Color32 {
        todo!()
    }
}
