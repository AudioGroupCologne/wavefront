use bevy::prelude::*;
use bevy_pixel_buffer::frame::GetFrameFromImages;
use bevy_pixel_buffer::pixel::Pixel;
use bevy_pixel_buffer::query::QueryPixelBuffer;
use egui::Color32;

use crate::components::microphone::Microphone;
use crate::components::states::Drag;
use crate::components::wall::{CircWall, RectWall, WResize, Wall};
use crate::grid::grid::Grid;
use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::transformations::{coords_to_index, map_range};
use crate::ui::state::{FftMicrophone, UiState};

#[derive(Resource)]
pub struct Gradient(pub Color32, pub Color32);

impl Gradient {
    pub fn new() -> Self {
        Self(Color32::from_rgb(0, 0, 0), Color32::from_rgb(255, 255, 255))
    }

    pub fn at(&self, percent: f32, contrast: f32) -> Color32 {
        let percent = percent * contrast;
        let result_red = self.0.r() as f32 + percent * (self.1.r() as f32 - self.0.r() as f32);
        let result_green = self.0.g() as f32 + percent * (self.1.g() as f32 - self.0.g() as f32);
        let result_blue = self.0.b() as f32 + percent * (self.1.b() as f32 - self.0.b() as f32);
        Color32::from_rgb(result_red as u8, result_green as u8, result_blue as u8)
    }
}

pub fn draw_pixels(
    pixel_buffers: QueryPixelBuffer,
    grid: Res<Grid>,
    gradient: Res<Gradient>,
    ui_state: Res<UiState>,
    fft_microphone: Res<FftMicrophone>,
    microphones: Query<&Microphone>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut items = query.iter();

    let abc_boundary_width = if ui_state.render_abc_area {
        0
    } else {
        ui_state.boundary_width
    };

    // draw TLM and walls
    let mut frame = images.frame(items.next().expect("one pixel buffer"));
    frame.per_pixel_par(|coords, _| {
        let current_index = coords_to_index(
            coords.x + abc_boundary_width,
            coords.y + abc_boundary_width,
            ui_state.boundary_width,
        );
        if grid.wall_cache[current_index].is_wall {
            let mut reflection_factor = grid.wall_cache[current_index].reflection_factor;
            if reflection_factor == 0. {
                reflection_factor = 1.;
            }
            return Pixel {
                r: (reflection_factor * 255.) as u8,
                g: (reflection_factor * 255.) as u8,
                b: (reflection_factor * 255.) as u8,
                a: 255,
            };
        }

        let p = grid.pressure[current_index];

        let color = gradient.at(p, ui_state.gradient_contrast);
        Pixel {
            r: color.r(),
            g: color.g(),
            b: color.b(),
            a: 255,
        }
    });

    // draw spectrum
    if ui_state.show_plots && fft_microphone.mic_id.is_some() {
        let mut frame = images.frame(items.next().expect("two pixel buffers"));

        // the mic that is selected might have been deleted, so we need to check if it still exists
        if let Some(mic) = microphones
            .iter()
            .find(|m| m.id == fft_microphone.mic_id.expect("no mic selected"))
        {
            // let spectrum = &mic.spectrum;
            // let len_y = spectrum.len();

            // frame.per_pixel_par(|coords, _| {
            //         let gray = if len_y > 1 && coords.y < len_y as u32 {
            //             spectrum[coords.y as usize]
            //                 //TODO: is 120 hardcoded <- doesn't work when frequency range changes and linear
            //                 //TODO: the spectrum is now log scaled, the spectrum does not consider this
            //                 [u32_map_range(0, (fft_microphone.spectrum_size.x) as u32, 0, 120, coords.x) as usize]
            //                 [1]
            //                 * 255.
            //         } else {
            //             0.
            //         } as u8;

            //         Pixel {
            //             r: gray,
            //             g: gray,
            //             b: gray,
            //             a: 255,
            //         }
            //     });

            // TODO: instead of the previous implementation, do the fft each frame (like in the freq plot)
            // and then write the result to the pixel buffer (and shift the previous values to the left)
            // this way we do not have to save all the values in a vec.

            // TODO: maybe reset the frame each time the mic changes

            let new_spectrum = crate::math::fft::calc_mic_spectrum(&mic);
            let frame_size = frame.size();

            // shift the old values to the left
            for y in 0..frame_size.y {
                for x in 0..frame_size.x - 1 {
                    let index = x + y * frame_size.x;
                    frame.raw_mut()[index as usize] = frame.raw()[(index + 1) as usize];
                }
            }

            // write the new values to the right
            let spectrum_len = new_spectrum.len();
            for y in 0..frame_size.y as usize {
                // TODO: log scale the y values
                let mapped_y = map_range(0, frame_size.y as usize, 0, spectrum_len, y);
                let gray = if spectrum_len > 1 && y < spectrum_len {
                    new_spectrum[mapped_y as usize][1] * 255.
                } else {
                    0.
                } as u8;

                let index = frame_size.x - 1 + y as u32 * frame_size.x;
                frame.raw_mut()[index as usize] = Pixel {
                    r: gray,
                    g: gray,
                    b: gray,
                    a: 255,
                };
            }
        }
    }
}

pub fn draw_overlays(
    pixel_buffers: QueryPixelBuffer,
    rect_walls_overlay: Query<&RectWall, Or<(With<WResize>, With<Drag>)>>,
    circ_walls_overlay: Query<&CircWall, Or<(With<WResize>, With<Drag>)>>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut frame = images.frame(query.iter().next().expect("one pixel buffer"));

    let raw_pixles = frame.raw_mut();

    for wall in rect_walls_overlay.iter() {
        for x in wall.rect.min.x..=wall.rect.max.x {
            for y in wall.rect.min.y..=wall.rect.max.y {
                let index = x + y * SIMULATION_WIDTH;

                let r = raw_pixles[index as usize].r;
                let g = raw_pixles[index as usize].g;
                let b = raw_pixles[index as usize].b;

                raw_pixles[index as usize] = Pixel {
                    r: map_range(0, 255, 80, 200, r as u32) as u8,
                    g: map_range(0, 255, 80, 200, g as u32) as u8,
                    b: map_range(0, 255, 80, 255, b as u32) as u8,
                    a: 255,
                };
            }
        }
    }

    for wall in circ_walls_overlay.iter() {
        if !wall.is_hollow {
            // center +- radius for smaller rect
            for x in 0..SIMULATION_WIDTH {
                for y in 0..SIMULATION_HEIGHT {
                    if wall.contains(x, y) {
                        let index = x + y * SIMULATION_WIDTH;

                        let r = raw_pixles[index as usize].r;
                        let g = raw_pixles[index as usize].g;
                        let b = raw_pixles[index as usize].b;

                        raw_pixles[index as usize] = Pixel {
                            r: map_range(0, 255, 80, 200, r as u32) as u8,
                            g: map_range(0, 255, 80, 200, g as u32) as u8,
                            b: map_range(0, 255, 80, 255, b as u32) as u8,
                            a: 255,
                        };
                    }
                }
            }
        }
    }

    for wall in circ_walls_overlay.iter() {
        let mut b_x = 0i32;
        let mut b_y = wall.radius as i32;
        let mut d = 1 - wall.radius as i32;
        while b_x <= b_y {
            for (x, y) in [
                (wall.center.x as i32 + b_x, wall.center.y as i32 + b_y),
                (wall.center.x as i32 + b_x, wall.center.y as i32 - b_y),
                (wall.center.x as i32 - b_x, wall.center.y as i32 + b_y),
                (wall.center.x as i32 - b_x, wall.center.y as i32 - b_y),
                (wall.center.x as i32 + b_y, wall.center.y as i32 + b_x),
                (wall.center.x as i32 + b_y, wall.center.y as i32 - b_x),
                (wall.center.x as i32 - b_y, wall.center.y as i32 + b_x),
                (wall.center.x as i32 - b_y, wall.center.y as i32 - b_x),
            ] {
                if x >= 0 && x < SIMULATION_WIDTH as i32 && y >= 0 && y < SIMULATION_HEIGHT as i32 {
                    let angle = if (x as i32 - wall.center.x as i32) > 0 {
                        ((y as i32 - wall.center.y as i32) as f32
                            / (x as i32 - wall.center.x as i32) as f32)
                            .atan()
                    } else {
                        180f32.to_radians()
                            - ((y as i32 - wall.center.y as i32) as f32
                                / (x as i32 - wall.center.x as i32) as f32)
                                .atan()
                                .abs()
                    };

                    if angle.abs() >= wall.open_circ_segment {
                        let index = x as u32 + y as u32 * SIMULATION_WIDTH;
                        let r = raw_pixles[index as usize].r;
                        let g = raw_pixles[index as usize].g;
                        let b = raw_pixles[index as usize].b;

                        raw_pixles[index as usize] = Pixel {
                            r: map_range(0, 255, 80, 150, r as u32) as u8,
                            g: map_range(0, 255, 80, 150, g as u32) as u8,
                            b: map_range(0, 255, 80, 255, b as u32) as u8,
                            a: 255,
                        };
                    }
                }
            }

            if d < 0 {
                d = d + 2 * b_x + 3;
                b_x += 1;
            } else {
                d = d + 2 * (b_x - b_y) + 5;
                b_x += 1;
                b_y -= 1;
            }
        }
    }
}
