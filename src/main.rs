use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::EguiPlugin;
use bevy_pixel_buffer::prelude::*;
use tlm_rs::components::{GameTicks, GradientResource, Microphone, Source};
use tlm_rs::constants::*;
use tlm_rs::grid::{apply_system, calc_system, update_system, Grid};
use tlm_rs::input::mouse_button_input;
use tlm_rs::render::{draw_egui, draw_pixels, draw_walls, UiState};

fn main() {
    let size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        // size: UVec2::new(SIMULATION_WIDTH + 2 * E_AL, SIMULATION_HEIGHT + 2 * E_AL), // render abc
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };

    let grid = Grid::default();

    let game_ticks = GameTicks::default();

    let gradient = GradientResource::with_custom();

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "TLM Demo in Rust".into(),
                    resolution: (1920., 1080.).into(),
                    ..default()
                }),
                ..default()
            }),
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
                PixelBufferBuilder::new()
                    .with_size(size)
                    .with_render(false)
                    .setup(),
                Source::spawn_initial_sources,
                Microphone::spawn_initial_microphones,
            ),
        )
        .add_systems(
            Update,
            (
                (calc_system, apply_system, update_system).chain(),
                (draw_pixels, draw_walls, draw_egui).chain(),
                mouse_button_input,
                bevy::window::close_on_esc,
            ),
        )
        .run();
}
