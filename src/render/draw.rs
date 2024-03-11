use bevy::prelude::*;
use bevy_pixel_buffer::frame::GetFrameFromImages;
use bevy_pixel_buffer::pixel::Pixel;
use bevy_pixel_buffer::query::QueryPixelBuffer;
use egui::Color32;

use crate::components::microphone::Microphone;
use crate::components::states::Overlay;
use crate::components::wall::{CircWall, RectWall, Wall};
use crate::grid::grid::Grid;
use crate::math::constants::SIMULATION_WIDTH;
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
    rect_walls: Query<&RectWall, Without<Overlay>>,
    circ_walls: Query<&CircWall, Without<Overlay>>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut items = query.iter();

    let boundary_width = if ui_state.render_abc_area {
        ui_state.boundary_width
    } else {
        0
    };

    // draw TLM and walls
    let mut frame = images.frame(items.next().expect("one pixel buffer"));
    frame.per_pixel_par(|coords, _| {
        for wall in rect_walls.iter() {
            if wall.contains(coords.x - boundary_width, coords.y - boundary_width) {
                return Pixel {
                    r: (wall.reflection_factor * 255.) as u8,
                    g: (wall.reflection_factor * 255.) as u8,
                    b: (wall.reflection_factor * 255.) as u8,
                    a: 255,
                };
            }
        }
        for wall in circ_walls.iter() {
            if wall.contains(coords.x - boundary_width, coords.y - boundary_width) {
                return Pixel {
                    r: (wall.reflection_factor * 255.) as u8,
                    g: (wall.reflection_factor * 255.) as u8,
                    b: (wall.reflection_factor * 255.) as u8,
                    a: 255,
                };
            }
        }

        let p = if ui_state.render_abc_area {
            grid.pressure[coords_to_index(coords.x, coords.y, ui_state.boundary_width)]
        } else {
            grid.pressure[coords_to_index(
                coords.x + ui_state.boundary_width,
                coords.y + ui_state.boundary_width,
                ui_state.boundary_width,
            )]
        };
        let color = gradient.at(p);
        Pixel {
            r: color.r(),
            g: color.g(),
            b: color.b(),
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
    rect_walls_overlay: Query<&RectWall, With<Overlay>>,
    circ_walls_overlay: Query<&CircWall, With<Overlay>>,
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

                raw_pixles[index as usize] = Pixel { r, g, b, a: 70 };
            }
        }
    }
}
