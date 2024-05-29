use bevy::prelude::*;
use bevy_file_dialog::DialogFileLoaded;
use serde::Deserialize;

use super::state::UiState;
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::{CircWall, RectWall};
use crate::events::UpdateWalls;
use crate::render::gradient::Gradient;
use crate::simulation::grid::Grid;
use crate::simulation::plugin::ComponentIDs;

/// Marker component for the file dialog and the corresponding event.
pub struct SaveFileContents;

/// The data that is loaded from a file. Used for deserialization.
#[derive(Deserialize)]
struct SaveData {
    sources: Vec<Source>,
    mics: Vec<Microphone>,
    rect_walls: Vec<RectWall>,
    circ_walls: Vec<CircWall>,
    gradient: Gradient,
    max_gradient: f32,
    min_gradient: f32,
}

/// Loads a file when receiving a [`DialogFileLoaded`] event from the file dialog.
/// All entities are despawned and the new entities are spawned.
pub fn file_loaded(
    mut ev_loaded: EventReader<DialogFileLoaded<SaveFileContents>>,
    mut commands: Commands,
    mut wall_update_ev: EventWriter<UpdateWalls>,
    mut grid: ResMut<Grid>,
    mut ids: ResMut<ComponentIDs>,
    mut gradient: ResMut<Gradient>,
    sources: Query<(Entity, &Source)>,
    mics: Query<(Entity, &Microphone)>,
    rect_walls: Query<(Entity, &RectWall)>,
    circ_walls: Query<(Entity, &CircWall)>,
    mut ui_state: ResMut<UiState>,
) {
    if let Some(data) = ev_loaded.read().next() {
        let save_data = serde_json::from_slice::<SaveData>(&data.contents).unwrap();

        // Clear all entities
        for (entity, _) in sources.iter() {
            commands.entity(entity).despawn();
        }
        for (entity, _) in mics.iter() {
            commands.entity(entity).despawn();
        }
        for (entity, _) in rect_walls.iter() {
            commands.entity(entity).despawn();
        }
        for (entity, _) in circ_walls.iter() {
            commands.entity(entity).despawn();
        }

        ids.reset();

        // Load entities
        for source in save_data.sources {
            commands.spawn(source);
            ids.get_new_source_id();
        }
        for mic in save_data.mics {
            commands.spawn(mic);
            ids.get_new_mic_id();
        }
        for rect_wall in save_data.rect_walls {
            commands.spawn(rect_wall);
            ids.get_new_wall_id();
        }
        for circ_wall in save_data.circ_walls {
            commands.spawn(circ_wall);
            ids.get_new_wall_id();
        }

        grid.reset_cells(ui_state.boundary_width);
        wall_update_ev.send(UpdateWalls);

        *gradient = save_data.gradient;
        ui_state.max_gradient = save_data.max_gradient;
        ui_state.min_gradient = save_data.min_gradient;
    }
}
