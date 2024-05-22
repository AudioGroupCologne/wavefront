use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_file_dialog::FileDialogPlugin;

use super::draw::draw_egui;
use super::loading::{file_loaded, SaveFileContents};
use super::state::{ClipboardBuffer, UiState};
use super::tabs::DockState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .init_resource::<ClipboardBuffer>()
            .init_resource::<DockState>()
            .add_plugins((
                FileDialogPlugin::new()
                    .with_save_file::<SaveFileContents>()
                    .with_load_file::<SaveFileContents>(),
                FrameTimeDiagnosticsPlugin,
            ))
            .add_systems(Update, (draw_egui, file_loaded));
    }
}
