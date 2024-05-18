use bevy::prelude::*;
use bevy_pixel_buffer::builder::PixelBufferBuilder;
use bevy_pixel_buffer::pixel_buffer::PixelBufferSize;

use super::draw::{draw_overlays, draw_pixels};
use super::gradient::{Gradient, GradientType};
use crate::math::constants::*;
use crate::ui::state::SimTime;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Gradient>()
            .init_resource::<SimTime>()
            .init_resource::<GradientType>()
            .add_systems(Startup, (setup_buffers,))
            .add_systems(Update, (draw_pixels, draw_overlays).chain());
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
