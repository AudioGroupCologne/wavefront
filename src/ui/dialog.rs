use bevy::prelude::*;
use bevy_file_dialog::DialogFileLoaded;
use serde::Deserialize;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::{CircWall, RectWall};

pub struct SaveFileContents;

#[derive(Deserialize)]
pub struct SaveData {
    pub sources: Vec<Source>,
    pub mics: Vec<Microphone>,
    pub rect_walls: Vec<RectWall>,
    pub circ_walls: Vec<CircWall>,
}

pub fn file_loaded(
    mut ev_loaded: EventReader<DialogFileLoaded<SaveFileContents>>,
    mut commands: Commands,
    sources: Query<(Entity, &Source)>,
    mics: Query<(Entity, &Microphone)>,
    rect_walls: Query<(Entity, &RectWall)>,
    circ_walls: Query<(Entity, &CircWall)>,
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

        // Load entities
        for source in save_data.sources {
            commands.spawn(source);
        }
        for mic in save_data.mics {
            commands.spawn(mic);
        }
        for rect_wall in save_data.rect_walls {
            commands.spawn(rect_wall);
        }
        for circ_wall in save_data.circ_walls {
            commands.spawn(circ_wall);
        }
    }
}
