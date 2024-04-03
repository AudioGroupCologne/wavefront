use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_pixel_buffer::bevy_egui::EguiPlugin;
use bevy_pixel_buffer::prelude::*;
use wavefront::events::EventPlugin;
use wavefront::input::plugin::InputPlugin;
use wavefront::render::plugin::RenderPlugin;
use wavefront::simulation::plugin::GridPlugin;
use wavefront::ui::plugin::UiPlugin;
use wavefront::undo::UndoPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "wavefront".into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
            PixelBufferPlugins,
            EguiPlugin,
            RenderPlugin,
            GridPlugin,
            InputPlugin,
            UiPlugin,
            EventPlugin,
            UndoPlugin,
        ))
        .run();
}
