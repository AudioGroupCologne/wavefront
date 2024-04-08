use bevy::app::{App, FixedUpdate, Plugin, Startup};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::Resource;

use super::grid::Grid;
use super::systems::{apply_system, calc_system, update_system};
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::math::constants::INIT_BOUNDARY_WIDTH;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        let mut grid = Grid::default();
        grid.cache_boundaries(INIT_BOUNDARY_WIDTH);

        app.insert_resource(grid)
            .init_resource::<ComponentIDs>()
            .add_systems(
                FixedUpdate,
                (calc_system, apply_system, update_system).chain(),
            );

        #[cfg(debug_assertions)]
        app.add_systems(
            Startup,
            (
                Source::spawn_initial_sources,
                Microphone::spawn_initial_microphones,
            ),
        );
    }
}

/// A resource to keep track of the current ids of objects in the grid
#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub struct ComponentIDs {
    current_mic_id: usize,
    current_source_id: usize,
    current_wall_id: usize,
}

impl ComponentIDs {
    /// Get a new **valid** id for a microphone
    pub fn get_new_mic_id(&mut self) -> usize {
        let current = self.current_mic_id;
        self.current_mic_id += 1;
        current
    }

    /// Get a new **valid** id for a source
    pub fn get_new_source_id(&mut self) -> usize {
        let current = self.current_source_id;
        self.current_source_id += 1;
        current
    }

    /// Get a new **valid** id for a wall
    pub fn get_new_wall_id(&mut self) -> usize {
        let current = self.current_wall_id;
        self.current_wall_id += 1;
        current
    }

    /// Decrements the current wall id
    pub fn decrement_wall_ids(&mut self) {
        self.current_wall_id -= 1;
    }

    /// Resets **all** ids to 0
    pub fn reset(&mut self) {
        self.current_mic_id = 0;
        self.current_source_id = 0;
        self.current_wall_id = 0;
    }
}
