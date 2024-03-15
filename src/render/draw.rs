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
use crate::math::transformations::{coords_to_index, u32_map_range};
use crate::ui::state::{PlotType, UiState};

#[derive(Resource)]
pub struct GradientResource(pub Color32, pub Color32);

impl GradientResource {
    pub fn new() -> Self {
        Self(Color32::from_rgb(0, 0, 0), Color32::from_rgb(255, 255, 255))
    }

    pub fn at(&self, percent: f32) -> Color32 {
        let result_red = self.0.r() as f32 + percent * (self.1.r() as f32 - self.0.r() as f32);
        let result_green = self.0.g() as f32 + percent * (self.1.g() as f32 - self.0.g() as f32);
        let result_blue = self.0.b() as f32 + percent * (self.1.b() as f32 - self.0.b() as f32);
        Color32::from_rgb(result_red as u8, result_green as u8, result_blue as u8)
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

        let color = gradient.at(p);
        Pixel {
            r: color.r(),
            g: color.g(),
            b: color.b(),
            a: 255,
        }
    });

    // draw spectrum
    if ui_state.plot_type == PlotType::FrequencyDomain && ui_state.current_fft_microphone.is_some()
    {
        let mut frame = images.frame(items.next().expect("two pixel buffers"));

        // the mic that is selected might have been deleted, so we need to check if it still exists
        if let Some(mic) = microphones
            .iter()
            .find(|m| m.id == ui_state.current_fft_microphone.expect("no mic selected")) {
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
                // no out of bounds check
                let index = x + y * SIMULATION_WIDTH;

                let r = raw_pixles[index as usize].r;
                let g = raw_pixles[index as usize].g;
                let b = raw_pixles[index as usize].b;

                raw_pixles[index as usize] = Pixel {
                    r: u32_map_range(0, 255, 80, 255, r as u32) as u8,
                    g: u32_map_range(0, 255, 80, 255, g as u32) as u8,
                    b: u32_map_range(0, 255, 80, 255, b as u32) as u8,
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
                        // no out of bounds check
                        let index = x + y * SIMULATION_WIDTH;

                        let r = raw_pixles[index as usize].r;
                        let g = raw_pixles[index as usize].g;
                        let b = raw_pixles[index as usize].b;

                        raw_pixles[index as usize] = Pixel {
                            r: u32_map_range(0, 255, 80, 255, r as u32) as u8,
                            g: u32_map_range(0, 255, 80, 255, g as u32) as u8,
                            b: u32_map_range(0, 255, 80, 255, b as u32) as u8,
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
                    let index = x as u32 + y as u32 * SIMULATION_WIDTH;
                    let r = raw_pixles[index as usize].r;
                    let g = raw_pixles[index as usize].g;
                    let b = raw_pixles[index as usize].b;

                    // make more visible
                    raw_pixles[index as usize] = Pixel {
                        r: u32_map_range(0, 255, 80, 255, r as u32) as u8,
                        g: u32_map_range(0, 255, 80, 255, g as u32) as u8,
                        b: u32_map_range(0, 255, 80, 255, b as u32) as u8,
                        a: 255,
                    };
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
