use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_file_dialog::FileDialogPlugin;

use super::draw::draw_egui;
use super::loading::{
    scene_save_file_loaded, wav_file_loaded, SceneSaveFileContents, WavFileContents,
};
use super::state::{ClipboardBuffer, FftMicrophone, UiState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .init_resource::<ClipboardBuffer>()
            .init_resource::<FftMicrophone>()
            .add_plugins((
                FileDialogPlugin::new()
                    .with_save_file::<SceneSaveFileContents>()
                    .with_load_file::<SceneSaveFileContents>()
                    .with_load_file::<WavFileContents>(),
                FrameTimeDiagnosticsPlugin,
            ))
            .add_systems(Update, (draw_egui, scene_save_file_loaded, wav_file_loaded));
    }
}
