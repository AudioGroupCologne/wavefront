use bevy::prelude::*;
use bevy_file_dialog::FileDialogPlugin;

use super::dialog::{file_loaded, SaveFileContents};
use super::draw::draw_egui;
use super::state::{ClipboardBuffer, FftMicrophone, UiState};
use super::tabs::DockState;
use crate::input::input::copy_paste_system;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .init_resource::<ClipboardBuffer>()
            .init_resource::<DockState>()
            .init_resource::<FftMicrophone>()
            .add_plugins(
                FileDialogPlugin::new()
                    .with_save_file::<SaveFileContents>()
                    .with_load_file::<SaveFileContents>(),
            )
            .add_systems(Update, (draw_egui, file_loaded, copy_paste_system));
    }
}
