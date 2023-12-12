use bevy::prelude::*;
use bevy_pixel_buffer::frame::GetFrameFromImages;
use bevy_pixel_buffer::pixel::Pixel;
use bevy_pixel_buffer::query::QueryPixelBuffer;

use super::state::UiState;
use crate::components::microphone::Microphone;
use crate::components::wall::Wall;
use crate::grid::grid::Grid;
use crate::math::transformations::{coords_to_index, index_to_coords, u32_map_range};

#[derive(Resource)]
pub struct GradientResource(pub colorgrad::Gradient);

impl GradientResource {
    pub fn with_custom() -> Self {
        Self(
            colorgrad::CustomGradient::new()
                .colors(&[
                    colorgrad::Color::from_rgba8(250, 172, 168, 255),
                    colorgrad::Color::from_rgba8(0, 0, 0, 255),
                    colorgrad::Color::from_rgba8(221, 214, 243, 255),
                ])
                .domain(&[-2.0, 2.0])
                .build()
                .unwrap(),
        )
    }
}

pub fn draw_pixels(
    pixel_buffers: QueryPixelBuffer,
    grid: Res<Grid>,
    gradient: Res<GradientResource>,
    ui_state: Res<UiState>,
    microphones: Query<&Microphone>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut items = query.iter();
    let mut frame = images.frame(items.next().expect("one pixel buffer"));

    frame.per_pixel_par(|coords, _| {
        let p = if ui_state.render_abc_area {
            grid.cells[coords_to_index(coords.x, coords.y, 8, ui_state.e_al)]
        } else {
            grid.cells[coords_to_index(
                coords.x + ui_state.e_al,
                coords.y + ui_state.e_al,
                8,
                ui_state.e_al,
            )]
        };
        let color = gradient.0.at((p) as f64);
        Pixel {
            r: (color.r * 255.) as u8,
            g: (color.g * 255.) as u8,
            b: (color.b * 255.) as u8,
            a: 255,
        }
    });

    let mut frame = images.frame(items.next().expect("two pixel buffers"));
    if ui_state.show_fft && ui_state.current_fft_microphone.is_some() {
        //? for now we don't draw the spectrum if no mic is selected, is this ok?
        // paint spectrum
        let mic = microphones
            .iter()
            .find(|m| m.id == ui_state.current_fft_microphone.expect("no mic selected"))
            .unwrap();
        let spectrum = &mic.spektrum;
        let len_y = spectrum.len();

        // if len_y > 0 {
        //     println!("{:?}", spectrum);
        // }

        frame.per_pixel_par(|coords, _| Pixel {
            r: (if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize][u32_map_range(0, 250, 0, 120, coords.x) as usize][1]
                    * 255.
            //TODO: is 120 hardcoded + 250 is hardcoded
            } else {
                0.
            }) as u8,
            g: (if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize][u32_map_range(0, 250, 0, 120, coords.x) as usize][1]
                    * 255.
            //TODO: is 120 hardcoded + 250 is hardcoded
            } else {
                0.
            }) as u8,
            b: (if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize][u32_map_range(0, 250, 0, 120, coords.x) as usize][1]
                    * 255.
            //TODO: is 120 hardcoded + 250 is hardcoded
            } else {
                0.
            }) as u8,
            a: 255,
        });
    }
}

pub fn draw_walls(pixel_buffers: QueryPixelBuffer, walls: Query<&Wall>, ui_state: Res<UiState>) {
    let (query, mut images) = pixel_buffers.split();
    let mut frame = images.frame(query.iter().next().expect("one pixel buffer"));

    for wall in walls.iter() {
        let (x, y) = index_to_coords(wall.0 as u32, ui_state.e_al);
        frame
            .set(
                UVec2::new(x, y),
                Pixel {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
            )
            .expect("Wall pixel out of bounds");
    }
}
