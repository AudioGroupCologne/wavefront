use bevy::prelude::*;
use bevy_file_dialog::FileDialogPlugin;

use super::dialog::{file_loaded, SaveFileContents};
use super::draw::draw_egui;
use super::state::UiState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .add_plugins(
                FileDialogPlugin::new()
                    .with_save_file::<SaveFileContents>()
                    .with_load_file::<SaveFileContents>(),
            )
            .add_systems(Update, (draw_egui, file_loaded));
    }
}
