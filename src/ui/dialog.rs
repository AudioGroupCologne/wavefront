use bevy::prelude::*;
use bevy_file_dialog::DialogFileLoaded;
use serde::Deserialize;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::Wall;

pub struct SaveFileContents;

#[derive(Deserialize)]
pub struct SaveData {
    pub sources: Vec<Source>,
    pub mics: Vec<Microphone>,
    pub walls: Vec<Wall>,
}

pub fn file_loaded(
    mut ev_loaded: EventReader<DialogFileLoaded<SaveFileContents>>,
    mut commands: Commands,
    sources: Query<(Entity, &Source)>,
    mics: Query<(Entity, &Microphone)>,
    walls: Query<(Entity, &Wall)>,
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
        for (entity, _) in walls.iter() {
            commands.entity(entity).despawn();
        }

        // Load entities
        for source in save_data.sources {
            commands.spawn(source);
        }
        for mic in save_data.mics {
            commands.spawn(mic);
        }
        for wall in save_data.walls {
            commands.spawn(wall);
        }
    }
}
