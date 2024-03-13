use egui::{Pos2, Rect};

use super::constants::*;
use crate::ui::state::UiState;

/// Calculates 1D array index from x,y coordinates (and an offset `index`)
pub fn coords_to_index(x: u32, y: u32, boundary_width: u32) -> usize {
    (y * (SIMULATION_WIDTH + 2 * boundary_width) + x) as usize
}

/// Calculates x, y coordinates from 1D array index
pub fn index_to_coords(i: u32, boundary_width: u32) -> (u32, u32) {
    let x = i % (SIMULATION_WIDTH + 2 * boundary_width);
    let y = i / (SIMULATION_WIDTH + 2 * boundary_width);
    (x, y)
}

/// Maps u32 s from range a1 to a2 to b1 to b2
pub fn u32_map_range(a1: u32, a2: u32, b1: u32, b2: u32, s: u32) -> u32 {
    b1 + ((s - a1) * (b2 - b1) / (a2 - a1))
}

/// Maps f32 s from range a1 to a2 to b1 to b2
pub fn f32_map_range(a1: f32, a2: f32, b1: f32, b2: f32, s: f32) -> f32 {
    b1 + ((s - a1) * (b2 - b1) / (a2 - a1))
}

/// converts screen coordinates to grid coordinates
pub fn screen_to_grid(x: f32, y: f32, image_rect: Rect, ui_state: &UiState) -> Option<(u32, u32)> {
    let boundary_width = if ui_state.render_abc_area {
        ui_state.boundary_width
    } else {
        0
    };

    let width = image_rect.width();
    let height = image_rect.height();
    let x = x - image_rect.min.x;
    let y = y - image_rect.min.y;

    if x >= width || y >= height || x < 0. || y < 0. {
        return None;
    }

    Some((
        u32_map_range(
            0,
            width as u32,
            0,
            SIMULATION_WIDTH + 2 * boundary_width,
            x as u32,
        ) - boundary_width,
        u32_map_range(
            0,
            height as u32,
            0,
            SIMULATION_HEIGHT + 2 * boundary_width,
            y as u32,
        ) - boundary_width,
    ))
}

/// grid position in 0..SIMULATION_WIDTH and 0..SIMULATION_HEIGHT
pub fn screen_to_nearest_grid(x: f32, y: f32, image_rect: Rect) -> Option<(u32, u32)> {
    let width = image_rect.width() as u32;
    let height = image_rect.height() as u32;

    let mut x = if (x as u32) < image_rect.min.x as u32 {
        0
    } else {
        x as u32 - image_rect.min.x as u32
    };
    let mut y = if (y as u32) < image_rect.min.y as u32 {
        0
    } else {
        y as u32 - image_rect.min.y as u32
    };

    x = if x > width { width } else { x };
    y = if y > height { height } else { y };

    Some((
        u32_map_range(0, width, 0, SIMULATION_WIDTH - 1, x),
        u32_map_range(0, height, 0, SIMULATION_HEIGHT - 1, y),
    ))
}

pub fn grid_to_image(pos: Pos2, image_rect: &Rect) -> Pos2 {
    Pos2::new(
        f32_map_range(
            0.,
            SIMULATION_WIDTH as f32,
            image_rect.min.x,
            image_rect.max.x,
            pos.x,
        ),
        f32_map_range(
            0.,
            SIMULATION_HEIGHT as f32,
            image_rect.min.y,
            image_rect.max.y,
            pos.y,
        ),
    )
}
