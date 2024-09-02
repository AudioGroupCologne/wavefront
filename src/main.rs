#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::WinitWindows;
use bevy_pixel_buffer::bevy_egui::EguiPlugin;
use bevy_pixel_buffer::prelude::*;
use wavefront::events::EventPlugin;
use wavefront::input::plugin::InputPlugin;
use wavefront::render::plugin::RenderPlugin;
use wavefront::simulation::plugin::GridPlugin;
use wavefront::ui::plugin::UiPlugin;
use wavefront::undo::UndoPlugin;
use winit::window::Icon;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

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
            PixelBufferPlugins,
            EguiPlugin,
            RenderPlugin,
            GridPlugin,
            InputPlugin,
            UiPlugin,
            EventPlugin,
            UndoPlugin,
        ))
        .add_systems(Startup, set_window_icon)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .run();
}

fn set_window_icon(windows: NonSend<WinitWindows>) {
    let (icon_rgba, icon_width, icon_height) = {
        let image = include_bytes!("../assets/icon.png");
        let image = image::load_from_memory(image)
            .expect("Failed to load icon from memory")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}
