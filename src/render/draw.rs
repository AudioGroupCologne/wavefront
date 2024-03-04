use bevy::prelude::*;
use bevy_pixel_buffer::frame::GetFrameFromImages;
use bevy_pixel_buffer::pixel::Pixel;
use bevy_pixel_buffer::query::QueryPixelBuffer;

use crate::components::microphone::Microphone;
use crate::components::states::Overlay;
use crate::components::wall::{WallBlock, WallCell};
use crate::grid::grid::Grid;
use crate::math::constants::SIMULATION_WIDTH;
use crate::math::transformations::{coords_to_index, u32_map_range};
use crate::ui::state::{PlotType, UiState};

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

    // draw TLM
    let mut frame = images.frame(items.next().expect("one pixel buffer"));
    frame.per_pixel_par(|coords, _| {
        let p = if ui_state.render_abc_area {
            grid.cells[coords_to_index(coords.x, coords.y, ui_state.e_al)]
        } else {
            grid.cells[coords_to_index(
                coords.x + ui_state.e_al,
                coords.y + ui_state.e_al,
                ui_state.e_al,
            )]
        }
        .pressure;
        let color = gradient.0.at((p) as f64);
        Pixel {
            r: (color.r * 255.) as u8,
            g: (color.g * 255.) as u8,
            b: (color.b * 255.) as u8,
            a: 255,
        }
    });

    // draw spectrum
    let mut frame = images.frame(items.next().expect("two pixel buffers"));
    if ui_state.plot_type == PlotType::FrequencyDomain && ui_state.current_fft_microphone.is_some()
    {
        let mic = microphones
            .iter()
            .find(|m| m.id == ui_state.current_fft_microphone.expect("no mic selected"))
            .unwrap();
        let spectrum = &mic.spectrum;
        let len_y = spectrum.len();

        frame.per_pixel_par(|coords, _| Pixel {
            //TODO: is 120 hardcoded <- doesn't work when frequency range changes and linear
            r: (if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize]
                    [u32_map_range(0, (ui_state.spectrum_size.x) as u32, 0, 120, coords.x) as usize]
                    [1]
                    * 255.
            } else {
                0.
            }) as u8,
            g: (if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize]
                    [u32_map_range(0, (ui_state.spectrum_size.x) as u32, 0, 120, coords.x) as usize]
                    [1]
                    * 255.
            } else {
                0.
            }) as u8,
            b: (if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize]
                    [u32_map_range(0, (ui_state.spectrum_size.x) as u32, 0, 120, coords.x) as usize]
                    [1]
                    * 255.
            } else {
                0.
            }) as u8,
            a: 255,
        });
    }
}

pub fn draw_wall_blocks(
    pixel_buffers: QueryPixelBuffer,
    walls: Query<&WallBlock, Without<Overlay>>,
    walls_overlay: Query<&WallBlock, With<Overlay>>,
    ui_state: Res<UiState>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut frame = images.frame(query.iter().next().expect("one pixel buffer"));
    for wall in walls.iter() {
        let min = wall.calc_rect.min;
        let x_sign = wall.calc_rect.width().signum();
        let y_sign = wall.calc_rect.height().signum();

        let offset = if ui_state.render_abc_area {
            ui_state.e_al
        } else {
            0
        };

        for x in 0..=wall.calc_rect.width().abs() as u32 {
            for y in 0..=wall.calc_rect.height().abs() as u32 {
                frame
                    .set(
                        UVec2::new(
                            (min.x + x as f32 * x_sign) as u32 + offset,
                            (min.y + y as f32 * y_sign) as u32 + offset,
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

    let raw_pixles = frame.raw_mut();

    for wall in walls_overlay.iter() {
        let min = wall.calc_rect.min;
        let x_sign = wall.calc_rect.width().signum();
        let y_sign = wall.calc_rect.height().signum();

        let offset = if ui_state.render_abc_area {
            ui_state.e_al
        } else {
            0
        };

        for x in 0..=wall.calc_rect.width().abs() as u32 {
            for y in 0..=wall.calc_rect.height().abs() as u32 {
                // no out of bounds check
                let index = ((min.x + x as f32 * x_sign) as u32 + offset)
                    + ((min.y + y as f32 * y_sign) as u32 + offset) * SIMULATION_WIDTH;

                let r = raw_pixles[index as usize].r;
                let g = raw_pixles[index as usize].g;
                let b = raw_pixles[index as usize].b;

                raw_pixles[index as usize] = Pixel { r, g, b, a: 70 };
            }
        }
    }
}

pub fn draw_wall_cells(
    pixel_buffers: QueryPixelBuffer,
    walls: Query<&WallCell>,
    ui_state: Res<UiState>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut frame = images.frame(query.iter().next().expect("one pixel buffer"));
    for wall in walls.iter() {
        let offset = if ui_state.render_abc_area {
            ui_state.e_al
        } else {
            0
        };

        frame
            .set(
                UVec2::new(wall.x + offset, wall.y + offset),
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
