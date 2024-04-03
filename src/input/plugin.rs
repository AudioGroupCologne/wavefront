use bevy::app::{App, Plugin, Update};

use super::systems::{button_input, copy_paste_system, event_input};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                button_input,
                event_input,
                copy_paste_system,
                bevy::window::close_on_esc,
            ),
        );
    }
}
