use bevy::prelude::*;
use bevy_pixel_buffer::pixel_buffer::PixelBufferSize;
use bevy_pixel_buffer::prelude::pixel_buffer_setup;

use super::draw::{draw_overlays, draw_pixels};
use super::gradient::Gradient;
use crate::math::constants::*;
use crate::ui::state::SimTime;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let main_size: PixelBufferSize = PixelBufferSize {
            size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
            pixel_size: UVec2::new(1, 1),
        };

        app.init_resource::<Gradient>()
            .init_resource::<SimTime>()
            .init_resource::<Gradient>()
            .add_systems(Startup, (pixel_buffer_setup(main_size),))
            .add_systems(Update, (draw_pixels, draw_overlays).chain());
    }
}
