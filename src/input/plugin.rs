use bevy::app::{App, Plugin, Update};

use super::events::{update_wall_event, UpdateWalls};
use super::input::button_input;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (button_input, bevy::window::close_on_esc, update_wall_event),
        );
        app.add_event::<UpdateWalls>();
    }
}
