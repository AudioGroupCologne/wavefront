use bevy::app::{App, Plugin, Update};
use bevy::ecs::schedule::IntoSystemConfigs;

use super::grid::Grid;
use super::systems::{apply_system, calc_system, update_system};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        let grid = Grid::default();

        app.insert_resource(grid)
            .add_systems(Update, (calc_system, apply_system, update_system).chain());
    }
}
