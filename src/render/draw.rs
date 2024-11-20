use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_pixel_buffer::frame::GetFrame;
use bevy_pixel_buffer::pixel::Pixel;
use bevy_pixel_buffer::query::QueryPixelBuffer;

use super::gradient::Gradient;
use crate::components::states::Move;
use crate::components::wall::{CircWall, RectWall, WResize, Wall};
use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::transformations::{coords_to_index, map_range};
use crate::simulation::grid::Grid;
use crate::ui::state::UiState;

pub fn draw_pixels(
    mut pixel_buffer: QueryPixelBuffer,
    grid: Res<Grid>,
    gradient: Res<Gradient>,
    ui_state: Res<UiState>,
) {
    let abc_boundary_width = if ui_state.render_abc_area {
        0
    } else {
        ui_state.boundary_width
    };

    // draw TLM and walls
    pixel_buffer.frame().per_pixel_par(|coords, _| {
        let current_index = coords_to_index(
            coords.x + abc_boundary_width,
            coords.y + abc_boundary_width,
            ui_state.boundary_width,
        );

        if current_index >= grid.wall_cache.len() {
            return Pixel {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            };
        }

        if grid.wall_cache[current_index].is_wall {
            let reflection_factor = grid.wall_cache[current_index].draw_reflection_factor;
            return Pixel {
                r: (reflection_factor * 255.) as u8,
                g: (reflection_factor * 255.) as u8,
                b: (reflection_factor * 255.) as u8,
                a: 255,
            };
        }

        let p = grid.pressure[current_index];

        let [r, g, b] = gradient.at(p, ui_state.min_gradient, ui_state.max_gradient);

        Pixel { r, g, b, a: 255 }
    });
}

type RectWallsResizeOrMove<'w, 's> =
    Query<'w, 's, &'static RectWall, Or<(With<WResize>, With<Move>)>>;
type CircWallsResizeOrMove<'w, 's> =
    Query<'w, 's, &'static CircWall, Or<(With<WResize>, With<Move>)>>;

pub fn draw_overlays(
    mut pixel_buffer: QueryPixelBuffer,
    rect_walls_overlay: RectWallsResizeOrMove,
    circ_walls_overlay: CircWallsResizeOrMove,
) {
    let mut frame = pixel_buffer.frame();

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
        } else {
            let mut b_x = 0i32;
            let mut b_y = wall.radius as i32;
            let mut d = 1 - wall.radius as i32;
            // TODO: don't name this var _thickness_, also it is in px
            let thickness = 5;
            while b_x <= b_y {
                for (x, y, t_x, t_y) in [
                    (
                        wall.center.x as i32 + b_x,
                        wall.center.y as i32 + b_y,
                        1,
                        -thickness,
                    ), // 0
                    (
                        wall.center.x as i32 + b_x,
                        wall.center.y as i32 - b_y,
                        1,
                        thickness,
                    ), // 1
                    (
                        wall.center.x as i32 - b_x,
                        wall.center.y as i32 + b_y,
                        1,
                        -thickness,
                    ), // 2
                    (
                        wall.center.x as i32 - b_x,
                        wall.center.y as i32 - b_y,
                        1,
                        thickness,
                    ), // 3
                    (
                        wall.center.x as i32 + b_y,
                        wall.center.y as i32 + b_x,
                        -thickness,
                        1,
                    ), // 4
                    (
                        wall.center.x as i32 + b_y,
                        wall.center.y as i32 - b_x,
                        -thickness,
                        1,
                    ), // 5
                    (
                        wall.center.x as i32 - b_y,
                        wall.center.y as i32 + b_x,
                        thickness,
                        1,
                    ), // 6
                    (
                        wall.center.x as i32 - b_y,
                        wall.center.y as i32 - b_x,
                        thickness,
                        1,
                    ), // 7
                ] {
                    if x >= 0
                        && x < SIMULATION_WIDTH as i32
                        && y >= 0
                        && y < SIMULATION_HEIGHT as i32
                    {
                        // angle in [0, 2pi)
                        let mut angle = if (y - wall.center.y as i32) <= 0 {
                            ((x as f32 - wall.center.x as f32) / wall.radius as f32).acos()
                        } else {
                            TAU - ((x as f32 - wall.center.x as f32) / wall.radius as f32).acos()
                        };

                        angle = (angle + wall.rotation_angle.to_radians()) % TAU;

                        if angle >= wall.open_circ_segment.to_radians() / 2.
                            && angle <= TAU - wall.open_circ_segment.to_radians() / 2.
                            || !wall.is_hollow
                        {
                            for cur_x in if t_x > 0 { 0..t_x } else { (t_x + 1)..1 } {
                                for cur_y in if t_y > 0 { 0..t_y } else { (t_y + 1)..1 } {
                                    let index =
                                        (x + cur_x) as u32 + (y + cur_y) as u32 * SIMULATION_WIDTH;
                                    let r = raw_pixles[index as usize].r;
                                    let g = raw_pixles[index as usize].g;
                                    let b = raw_pixles[index as usize].b;

                                    raw_pixles[index as usize] = Pixel {
                                        r: map_range(0, 255, 100, 175, r as u32) as u8,
                                        g: map_range(0, 255, 80, 80, g as u32) as u8,
                                        b: map_range(0, 255, 80, 80, b as u32) as u8,
                                        a: 255,
                                    };
                                }
                            }
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
}
