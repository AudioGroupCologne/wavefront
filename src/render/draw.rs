use bevy::prelude::*;
use bevy_pixel_buffer::frame::GetFrameFromImages;
use bevy_pixel_buffer::pixel::Pixel;
use bevy_pixel_buffer::query::QueryPixelBuffer;

use crate::components::microphone::Microphone;
use crate::components::states::Overlay;
use crate::components::wall::{CircWall, RectWall, Wall};
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
        let p = if ui_state.render_abc_area {
            grid.pressure[coords_to_index(coords.x, coords.y, ui_state.boundary_width)]
        } else {
            grid.pressure[coords_to_index(
                coords.x + ui_state.boundary_width,
                coords.y + ui_state.boundary_width,
                ui_state.boundary_width,
            )]
        };
        let mut color = gradient.0.at((p) as f64);
        for wall in rect_walls.iter() {
            if wall.contains(coords.x - boundary_width, coords.y - boundary_width) {
                color.r = wall.reflection_factor as f64;
                color.g = wall.reflection_factor as f64;
                color.b = wall.reflection_factor as f64;
            }
        }
        for wall in circ_walls.iter() {
            if wall.contains(coords.x - boundary_width, coords.y - boundary_width) {
                color.r = wall.reflection_factor as f64;
                color.g = wall.reflection_factor as f64;
                color.b = wall.reflection_factor as f64;
            }
        }
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
    rect_walls_overlay: Query<&RectWall, With<Overlay>>,
    circ_walls_overlay: Query<&CircWall, With<Overlay>>,
) {
    let (query, mut images) = pixel_buffers.split();
    let mut frame = images.frame(query.iter().next().expect("one pixel buffer"));

    let raw_pixles = frame.raw_mut();

    for wall in rect_walls_overlay.iter() {
        // for x in wall.draw_rect.min.x..=wall.draw_rect.max.x {
        //     for y in wall.draw_rect.min.y..=wall.draw_rect.max.y {
        //         // no out of bounds check
        //         let index = x + y * SIMULATION_WIDTH;

        //         let r = raw_pixles[index as usize].r;
        //         let g = raw_pixles[index as usize].g;
        //         let b = raw_pixles[index as usize].b;

        //         raw_pixles[index as usize] = Pixel { r, g, b, a: 70 };
        //     }
        // }
    }
}
