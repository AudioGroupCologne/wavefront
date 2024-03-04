use bevy::prelude::*;
use bevy_file_dialog::FileDialogPlugin;
use bevy_pixel_buffer::builder::PixelBufferBuilder;
use bevy_pixel_buffer::pixel_buffer::PixelBufferSize;

use super::dialog::file_loaded;
use super::draw::{draw_pixels, draw_wall_blocks, draw_wall_cells, GradientResource};
use super::state::{GameTicks, UiState};
use super::ui::draw_egui;
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::math::constants::*;
use crate::render::dialog::SaveFileContents;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let game_ticks = GameTicks::default();

        let gradient = GradientResource::with_custom();

        app.insert_resource(gradient)
            .insert_resource(game_ticks)
            .init_resource::<UiState>()
            .add_plugins(
                FileDialogPlugin::new()
                    .with_save_file::<SaveFileContents>()
                    .with_load_file::<SaveFileContents>(),
            )
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
                    (draw_pixels, draw_wall_blocks, draw_wall_cells, draw_egui).chain(),
                    file_loaded,
                ),
            );
    }
}

pub fn setup_buffers(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let main_size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };
    let spectrum_size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(250, 500), // random init values
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };
    insert_pixel_buffer(&mut commands, &mut images, main_size); //main
    insert_pixel_buffer(&mut commands, &mut images, spectrum_size); //spectrum
}

fn insert_pixel_buffer(commands: &mut Commands, images: &mut Assets<Image>, size: PixelBufferSize) {
    PixelBufferBuilder::new()
        .with_render(false)
        .with_size(size)
        .spawn(commands, images);
}

fn setup_window(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.set_maximized(true);
}
