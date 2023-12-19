use bevy::prelude::*;
use bevy_pixel_buffer::frame::GetFrameFromImages;
use bevy_pixel_buffer::pixel::Pixel;
use bevy_pixel_buffer::query::QueryPixelBuffer;

use super::state::UiState;
use crate::components::microphone::Microphone;
use crate::components::wall::WallBlock;
use crate::grid::grid::Grid;
use crate::math::transformations::{coords_to_index, u32_map_range};

#[derive(Resource)]
pub struct GradientResource(pub colorgrad::Gradient);

impl GradientResource {
    pub fn with_custom() -> Self {
        Self(
            colorgrad::CustomGradient::new()
                .colors(&[
                    colorgrad::Color::from_rgba8(80, 80, 80, 255),
                    colorgrad::Color::from_rgba8(0, 0, 0, 255),
                    colorgrad::Color::from_rgba8(255, 255, 255, 255),
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

pub fn draw_walls(
    pixel_buffers: QueryPixelBuffer,
    walls: Query<&WallBlock>,
    ui_state: Res<UiState>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut frame = images.frame(query.iter().next().expect("one pixel buffer"));
    for wall in walls.iter() {
        let origin = wall.rect.min;
        let x_sign = wall.rect.width().signum();
        let y_sign = wall.rect.height().signum();

        let offset = if ui_state.render_abc_area {
            ui_state.e_al
        } else {
            0
        };

        for x in 0..wall.rect.width().abs() as u32 {
            for y in 0..wall.rect.height().abs() as u32 {
                frame
                    .set(
                        UVec2::new(
                            (origin.x + x as f32 * x_sign) as u32 + offset,
                            (origin.y + y as f32 * y_sign) as u32 + offset,
                        ),
                        Pixel {
                            r: (255. * wall.reflection_factor) as u8,
                            g: (255. * wall.reflection_factor) as u8,
                            b: (255. * wall.reflection_factor) as u8,
                            a: 255,
                        },
                    )
                    .expect("Wall pixel out of bounds");
            }
        }
    }
}
