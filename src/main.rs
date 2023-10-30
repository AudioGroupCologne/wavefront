use bevy::prelude::*;
use bevy_pixel_buffer::prelude::*;
use tlm_rs::components::{GradientResource, Source};
use tlm_rs::constants::*;
use tlm_rs::grid::{apply_system, calc_system, update_system, Grid};
use tlm_rs::input::mouse_button_input;
use tlm_rs::render::draw_pixels;

fn main() {
    let size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };

    let grid = Grid::default();

    let gradient = GradientResource::with_custom();

    App::new()
        .add_plugins((DefaultPlugins, PixelBufferPlugin))
        .insert_resource(grid)
        .insert_resource(gradient)
        .add_systems(
            Startup,
            (pixel_buffer_setup(size), Source::spawn_initial_sources),
        )
        .add_systems(Update, (bevy::window::close_on_esc, mouse_button_input))
        .add_systems(
            Update,
            (
                (calc_system, apply_system, update_system).chain(),
                draw_pixels,
            ),
        )
        .run();
}
