use bevy::app::{App, Plugin, Update};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::Resource;

use super::grid::Grid;
use super::systems::{apply_system, calc_system, update_system};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        let grid = Grid::default();

        let component_ids = ComponentIDs::new();

        app.insert_resource(grid)
            .insert_resource(component_ids)
            .add_systems(Update, (calc_system, apply_system, update_system).chain());
    }
}

#[derive(Resource)]
pub struct ComponentIDs {
    microphone_id: usize,
}

impl ComponentIDs {
    pub fn new() -> Self {
        Self { microphone_id: 0 }
    }
    pub fn get_current_mic_id(&mut self) -> usize {
        let current = self.microphone_id;
        self.microphone_id += 1;
        current
    }
}
