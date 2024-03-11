use bevy::prelude::*;
use bevy_pixel_buffer::builder::PixelBufferBuilder;
use bevy_pixel_buffer::pixel_buffer::PixelBufferSize;

use super::draw::{draw_pixels, draw_walls, GradientResource};
use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::math::constants::*;
use crate::ui::state::GameTicks;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let game_ticks = GameTicks::default();

        let gradient = GradientResource::with_custom();

        app.insert_resource(gradient)
            .insert_resource(game_ticks)
            .add_systems(
                Startup,
                (
                    setup_buffers,
                    setup_window,
                    Source::spawn_initial_sources,
                    Microphone::spawn_initial_microphones,
                ),
            )
            .add_systems(Update, (draw_pixels, draw_walls).chain());
    }
}

pub fn setup_buffers(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let main_size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        pixel_size: UVec2::new(1, 1),
    };
    let spectrum_size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(250, 500), // random init values
        pixel_size: UVec2::new(1, 1),
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
