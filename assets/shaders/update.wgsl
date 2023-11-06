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

    // let cur_x = (workgroup_id.x + 1u) * local_id.x;
    // let cur_y = (workgroup_id.y + 1u) * local_id.y;

    // let array_pos = coords_to_index(cur_x, cur_y, 0u);

    let array_pos = coords_to_index(global_id.x, global_id.y, 0u);

    grid[array_pos] = grid[array_pos + 4u];
    grid[array_pos + 1u] = grid[array_pos + 5u];
    grid[array_pos + 2u] = grid[array_pos + 6u];
    grid[array_pos + 3u] = grid[array_pos + 7u];

    //calculate pressure
    grid[array_pos + 8u] = 0.5 * (grid[array_pos] + grid[array_pos + 1u] + grid[array_pos + 2u] + grid[array_pos + 3u]);
}

fn coords_to_index(x: u32, y: u32, index: u32) -> u32 {
    return y * SIMULATION_WIDTH * NUM_INDEX + x * NUM_INDEX + index;
}