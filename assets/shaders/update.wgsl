@group(0) @binding(0)
var<uniform> NUM_INDEX: u32;

@group(0) @binding(1)
var<storage, read_write> grid: array<f32>;


@compute @workgroup_size(1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let array_pos = global_id.x * global_id.y * NUM_INDEX;

    grid[array_pos] = grid[array_pos + 4u];
    grid[array_pos + 1u] = grid[array_pos + 5u];
    grid[array_pos + 2u] = grid[array_pos + 6u];
    grid[array_pos + 3u] = grid[array_pos + 7u];

    //calculate pressure
    grid[array_pos + 8u] = 0.5 * (grid[array_pos] + grid[array_pos + 1u] + grid[array_pos + 2u] + grid[array_pos + 3u]);
}
