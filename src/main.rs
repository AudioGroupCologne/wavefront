use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::EguiPlugin;
use bevy_pixel_buffer::prelude::*;
use tlm_rs::components::{GameTicks, GradientResource, Microphone, Source};
use tlm_rs::grid::{apply_system, calc_system, update_system, Grid};
use tlm_rs::input::button_input;
// use tlm_rs::render::{draw_egui, draw_pixels, draw_walls, UiState};
use tlm_rs::render::{draw_egui, draw_pixels, setup_buffers, UiState};
fn main() {
    let grid = Grid::default();

    let game_ticks = GameTicks::default();

    let gradient = GradientResource::with_custom();

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "TLM Demo in Rust".into(),
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
            PixelBufferPlugins,
            EguiPlugin,
        ))
        .insert_resource(grid)
        .insert_resource(gradient)
        .insert_resource(game_ticks)
        .init_resource::<UiState>()
        .add_systems(
            Startup,
            (
                setup_buffers,
                setup_window,
                Source::spawn_initial_sources,
                Microphone::spawn_initial_microphones,
            ),
        )
        .add_systems(
            Update,
            (
                (calc_system, apply_system, update_system).chain(),
                // (draw_pixels, draw_walls, draw_egui).chain(),
                (draw_pixels, draw_egui).chain(),
                button_input,
                bevy::window::close_on_esc,
            ),
        )
        .run();
}

fn setup_window(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.set_maximized(true);
}
