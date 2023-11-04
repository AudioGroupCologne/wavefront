@group(0) @binding(0)
var<uniform> SIMULATION_WIDTH: u32;
@group(0) @binding(1)
var<uniform> SIMULATION_HEIGHT: u32;
@group(0) @binding(2)
var<uniform> NUM_INDEX: u32;

@group(0) @binding(3)
var<storage, read_write> grid: array<f32>;

//TODO: make this an array of structs
@group(0) @binding(4)
var<storage, read_write> sources: array<Source>;

// @group(0) @binding(2)
// var<storage, read_write> wall_pos: array<u32>;

struct Source {
    index: u32,
    value: f32,
}

@compute @workgroup_size(1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    // UPDATE
    for (var i: u32 = 0u; i < (SIMULATION_WIDTH * SIMULATION_HEIGHT); i++) {
        let array_pos = i * NUM_INDEX;

        grid[array_pos] = grid[array_pos + 4u];
        grid[array_pos + 1u] = grid[array_pos + 5u];
        grid[array_pos + 2u] = grid[array_pos + 6u];
        grid[array_pos + 3u] = grid[array_pos + 7u];

        //calculate pressure
        grid[array_pos + 8u] = 0.5 * (grid[array_pos] + grid[array_pos + 1u] + grid[array_pos + 2u] + grid[array_pos + 3u]);
    }

    // CALC
    for (var x: u32 = 0u; x < (SIMULATION_WIDTH - 1u); x++) {
        for (var y: u32 = 0u; y < (SIMULATION_HEIGHT - 1u); y++) {

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
    }

    // SOURCES
    for (var i: u32 = 0u; i < arrayLength(&sources); i++) {
        grid[sources[i].index + 4u] = sources[i].value;
        grid[sources[i].index + 5u] = sources[i].value;
        grid[sources[i].index + 6u] = sources[i].value;
        grid[sources[i].index + 7u] = sources[i].value;
    }
}

fn coords_to_index(x: u32, y: u32, index: u32) -> u32 {
    return y * SIMULATION_WIDTH * NUM_INDEX + x * NUM_INDEX + index;
}