use std::env;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_pixel_buffer::bevy_egui::EguiPlugin;
use bevy_pixel_buffer::prelude::*;
use tlm_rs::grid::plugin::GridPlugin;
use tlm_rs::input::plugin::InputPlugin;
use tlm_rs::render::plugin::RenderPlugin;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "TLM Demo in Rust".into(),
                    present_mode: PresentMode::AutoNoVsync,
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
        ))
        .run();
}
