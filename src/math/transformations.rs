use super::constants::*;

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
