use bevy::prelude::*;
use bevy_pixel_buffer::frame::GetFrameFromImages;
use bevy_pixel_buffer::pixel::Pixel;
use bevy_pixel_buffer::query::QueryPixelBuffer;

use crate::components::microphone::Microphone;
use crate::components::states::Overlay;
use crate::components::wall::{Wall, WallType};
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

        frame.per_pixel_par(|coords, _| {
            let gray = if len_y > 1 && coords.y < len_y as u32 {
                spectrum[coords.y as usize]
                    //TODO: is 120 hardcoded <- doesn't work when frequency range changes and linear
                    [u32_map_range(0, (ui_state.spectrum_size.x) as u32, 0, 120, coords.x) as usize]
                    [1]
                    * 255.
            } else {
                0.
            } as u8;

            Pixel {
                r: gray,
                g: gray,
                b: gray,
                a: 255,
            }
        });
    }
}

pub fn draw_walls(
    pixel_buffers: QueryPixelBuffer,
    walls: Query<&Wall, Without<Overlay>>,
    walls_overlay: Query<&Wall, With<Overlay>>,
    ui_state: Res<UiState>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut frame = images.frame(query.iter().next().expect("one pixel buffer"));
    let boundary_width = if ui_state.render_abc_area {
        ui_state.e_al
    } else {
        0
    };
    for wall in walls.iter() {
        match &wall.wall_type {
            WallType::Rectangle => {
                // this feels a bit sloppy
                if !wall.hollow {
                    for x in wall.draw_rect.min.x..=wall.draw_rect.max.x {
                        for y in wall.draw_rect.min.y..=wall.draw_rect.max.y {
                            frame
                                .set(
                                    UVec2::new(x + boundary_width, y + boundary_width),
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
                } else {
                    for x in wall.draw_rect.min.x..=wall.draw_rect.max.x {
                        frame
                            .set(
                                UVec2::new(
                                    x + boundary_width,
                                    wall.draw_rect.min.y + boundary_width,
                                ),
                                Pixel {
                                    r: (255. * wall.reflection_factor) as u8,
                                    g: (255. * wall.reflection_factor) as u8,
                                    b: (255. * wall.reflection_factor) as u8,
                                    a: 255,
                                },
                            )
                            .expect("Wall pixel out of bounds");
                        frame
                            .set(
                                UVec2::new(
                                    x + boundary_width,
                                    wall.draw_rect.max.y + boundary_width,
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
                    for y in wall.draw_rect.min.y..=wall.draw_rect.max.y {
                        frame
                            .set(
                                UVec2::new(
                                    wall.draw_rect.min.x + boundary_width,
                                    y + boundary_width,
                                ),
                                Pixel {
                                    r: (255. * wall.reflection_factor) as u8,
                                    g: (255. * wall.reflection_factor) as u8,
                                    b: (255. * wall.reflection_factor) as u8,
                                    a: 255,
                                },
                            )
                            .expect("Wall pixel out of bounds");
                        frame
                            .set(
                                UVec2::new(
                                    wall.draw_rect.max.x + boundary_width,
                                    y + boundary_width,
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
            WallType::Circle => todo!(),
        }
    }

    let raw_pixles = frame.raw_mut();

    for wall in walls_overlay.iter() {
        match &wall.wall_type {
            WallType::Rectangle => {
                for x in wall.draw_rect.min.x..=wall.draw_rect.max.x {
                    for y in wall.draw_rect.min.y..=wall.draw_rect.max.y {
                        // no out of bounds check
                        let index = x + y * SIMULATION_WIDTH;

                        let r = raw_pixles[index as usize].r;
                        let g = raw_pixles[index as usize].g;
                        let b = raw_pixles[index as usize].b;

                        raw_pixles[index as usize] = Pixel { r, g, b, a: 70 };
                    }
                }
            }
            WallType::Circle => todo!(),
        }
    }
}
