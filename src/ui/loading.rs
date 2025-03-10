use bevy::prelude::*;
use bevy_file_dialog::DialogFileLoaded;
use serde::Deserialize;

use super::state::UiState;
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::{CircWall, RectWall};
use crate::events::{Reset, UpdateWalls};
use crate::math::constants::PROPAGATION_SPEED;
use crate::render::gradient::Gradient;
use crate::simulation::grid::Grid;
use crate::simulation::plugin::{ComponentIDs, WaveSamples};

/// Marker component for the file dialog and the corresponding event.
pub struct SceneSaveFileContents;

/// The data that is loaded from a scene save file. Used for deserialization.
#[derive(Deserialize)]
struct SceneSaveData {
    sources: Vec<Source>,
    mics: Vec<Microphone>,
    rect_walls: Vec<RectWall>,
    circ_walls: Vec<CircWall>,
    gradient: Gradient,
    max_gradient: f32,
    min_gradient: f32,
    reset_on_change: bool,
    delta_l: f32,
}

/// Loads a file when receiving a [`DialogFileLoaded`] event from the file dialog.
/// All entities are despawned and the new entities are spawned.
pub fn scene_save_file_loaded(
    mut ev_loaded: EventReader<DialogFileLoaded<SceneSaveFileContents>>,
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
        let save_data = serde_json::from_slice::<SceneSaveData>(&data.contents).unwrap();

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

        *gradient = save_data.gradient;
        ui_state.max_gradient = save_data.max_gradient;
        ui_state.min_gradient = save_data.min_gradient;
        ui_state.reset_on_change = save_data.reset_on_change;
        ui_state.delta_l = save_data.delta_l;

        grid.reset_cells(ui_state.boundary_width);
        wall_update_ev.send(UpdateWalls);
    }
}

/// Marker component for the file dialog and the corresponding event.
pub struct WavFileContents;

pub fn wav_file_loaded(
    mut ev_loaded: EventReader<DialogFileLoaded<WavFileContents>>,
    mut ui_state: ResMut<UiState>,
    mut reset_ev: EventWriter<Reset>,
    mut wave_samples: ResMut<WaveSamples>,
) {
    if let Some(data) = ev_loaded.read().next() {
        // let reader = hound::WavReader::open("assets/misc/audio.wav");
        let reader = hound::WavReader::new(data.contents.as_slice());
        if reader.is_ok() {
            let mut reader = reader.unwrap();
            println!("{:?}", reader.spec().bits_per_sample);
            let samples = match reader.spec().sample_format {
                hound::SampleFormat::Int => {
                    match reader.spec().bits_per_sample {
                        16 => reader
                            .samples::<i16>()
                            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
                            .collect::<Vec<f32>>(),
                        32 => reader
                            .samples::<i32>()
                            .map(|s| s.unwrap() as f32 / i32::MAX as f32)
                            .collect::<Vec<f32>>(), //normalisation isn't correct i think
                        _ => todo!(),
                    }
                }
                hound::SampleFormat::Float => match reader.spec().bits_per_sample {
                    32 => reader
                        .samples::<f32>()
                        .collect::<Result<Vec<f32>, _>>()
                        .unwrap(),
                    _ => todo!(),
                },
            };
            wave_samples.0 = samples;

            // set delta l to correct sample rate
            ui_state.delta_l = PROPAGATION_SPEED / reader.spec().sample_rate as f32;

            reset_ev.send(Reset::default());
        }
    }
}
