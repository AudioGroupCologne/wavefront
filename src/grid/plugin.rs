use bevy::app::{App, Plugin, Update};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::Resource;

use super::grid::Grid;
use super::systems::{apply_system, calc_system, update_system};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        let grid = Grid::default();

        let component_ids = ComponentIDs::default();

        app.insert_resource(grid)
            .insert_resource(component_ids)
            .add_systems(Update, (calc_system, apply_system, update_system).chain());
    }
}

#[derive(Resource)]
pub struct ComponentIDs {
    microphone_id: usize,
    source_id: usize,
    wall_id: usize,
}

impl ComponentIDs {
    pub fn new() -> Self {
        Self {
            microphone_id: 0,
            source_id: 0,
            wall_id: 0,
        }
    }
    pub fn get_current_mic_id(&mut self) -> usize {
        let current = self.microphone_id;
        self.microphone_id += 1;
        current
    }
    pub fn get_current_source_id(&mut self) -> usize {
        let current = self.source_id;
        self.source_id += 1;
        current
    }
    pub fn get_current_wall_id(&mut self) -> usize {
        let current = self.wall_id;
        self.wall_id += 1;
        current
    }
}

impl Default for ComponentIDs {
    fn default() -> Self {
        Self::new()
    }
}
