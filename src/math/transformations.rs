use bevy_pixel_buffer::bevy_egui::egui::emath::Rect;
use bevy_pixel_buffer::bevy_egui::egui::Pos2;

use super::constants::*;
use crate::render::state::UiState;

/// Calculates 1D array index from x,y coordinates (and an offset `index`)
pub fn coords_to_index(x: u32, y: u32, index: u32, e_al: u32) -> usize {
    (y * (SIMULATION_WIDTH + 2 * e_al) * NUM_INDEX + x * NUM_INDEX + index) as usize
}

/// Calculates x,y coordinates from 1D array index
pub fn index_to_coords(i: u32, e_al: u32) -> (u32, u32) {
    let x = (i / 9) % (SIMULATION_WIDTH + 2 * e_al);
    let y = i / 9 / (SIMULATION_WIDTH + 2 * e_al);
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
        ui_state.e_al
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

pub fn true_rect_from_rect(rect: Rect) -> Rect {
    //Q1
    if rect.min.x < rect.max.x && rect.min.y > rect.max.y {
        return Rect {
            min: Pos2::new(rect.min.x, rect.max.y),
            max: Pos2::new(rect.max.x, rect.min.y),
        };
    }
    //Q2
    if rect.min.x > rect.max.x && rect.min.y > rect.max.y {
        return Rect {
            min: Pos2::new(rect.max.x, rect.max.y),
            max: Pos2::new(rect.min.x, rect.min.y),
        };
    }
    //Q3
    if rect.min.x > rect.max.x && rect.min.y < rect.max.y {
        return Rect {
            min: Pos2::new(rect.max.x, rect.min.y),
            max: Pos2::new(rect.min.x, rect.max.y),
        };
    }
    //Q4
    rect
}

pub fn screen_to_nearest_grid(
    x: f32,
    y: f32,
    image_rect: Rect,
    ui_state: &UiState,
) -> Option<(u32, u32)> {
    let boundary_width = if ui_state.render_abc_area {
        ui_state.e_al
    } else {
        0
    };

    let width = image_rect.width();
    let height = image_rect.height();
    let x = x - image_rect.min.x;
    let y = y - image_rect.min.y;

    if y >= height && x <= width && x > 0. {
        return Some((
            u32_map_range(
                0,
                width as u32,
                0,
                SIMULATION_WIDTH + 2 * boundary_width,
                x as u32,
            ) - boundary_width,
            SIMULATION_HEIGHT - 1,
        ));
    } else if x >= width && y <= height && y > 0. {
        return Some((
            SIMULATION_WIDTH - 1,
            u32_map_range(
                0,
                height as u32,
                0,
                SIMULATION_HEIGHT + 2 * boundary_width,
                y as u32,
            ) - boundary_width,
        ));
    } else if x < 0. && y <= height && y > 0. {
        return Some((
            0,
            u32_map_range(
                0,
                height as u32,
                0,
                SIMULATION_HEIGHT + 2 * boundary_width,
                y as u32,
            ) - boundary_width,
        ));
    } else if y < 0. && x <= width && x > 0. {
        return Some((
            u32_map_range(
                0,
                width as u32,
                0,
                SIMULATION_WIDTH + 2 * boundary_width,
                x as u32,
            ) - boundary_width,
            0,
        ));
    }

    Some((
        (u32_map_range(
            0,
            width as u32,
            0,
            SIMULATION_WIDTH + 2 * boundary_width,
            x as u32,
        ) - boundary_width)
            .clamp(0, SIMULATION_WIDTH - 1),
        (u32_map_range(
            0,
            height as u32,
            0,
            SIMULATION_HEIGHT + 2 * boundary_width,
            y as u32,
        ) - boundary_width)
            .clamp(0, SIMULATION_HEIGHT - 1),
    ))
}
