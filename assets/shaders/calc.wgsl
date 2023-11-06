@group(0) @binding(0)
var<uniform> SIMULATION_WIDTH: u32;

@group(0) @binding(1)
var<uniform> NUM_INDEX: u32;

@group(0) @binding(2)
var<storage, read_write> grid: array<f32>;

@compute @workgroup_size(8, 8)
// @compute @workgroup_size(1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    // let x = (workgroup_id.x + 1u) * local_id.x;
    // let y = (workgroup_id.y + 1u) * local_id.y;

    let x = global_id.x;
    let y = global_id.y;

    // CALC WITH WORKGROUPS
    let bottom_top = grid[coords_to_index(x, y + 1u, 2u)];
    let left_right = grid[coords_to_index(x - 1u, y, 3u)];
    let top_bottom = grid[coords_to_index(x, y - 1u, 0u)];
    let right_left = grid[coords_to_index(x + 1u, y, 1u)];

    let index = coords_to_index(x, y, 0u);

    grid[index + 4u] = 0.5 * (-bottom_top + left_right + top_bottom + right_left);
    grid[index + 5u] = 0.5 * (bottom_top - left_right + top_bottom + right_left);
    grid[index + 6u] = 0.5 * (bottom_top + left_right - top_bottom + right_left);
    grid[index + 7u] = 0.5 * (bottom_top + left_right + top_bottom - right_left);
}

fn coords_to_index(x: u32, y: u32, index: u32) -> u32 {
    return y * SIMULATION_WIDTH * NUM_INDEX + x * NUM_INDEX + index;
}