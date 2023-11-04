
@group(0) @binding(0)
var<storage, read_write> grid: array<f32>;

@group(0) @binding(1)
var<storage, read_write> source_pos: array<u32>;

@group(0) @binding(2)
var<storage, read_write> wall_pos: array<u32>;

@compute @workgroup_size(1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
    @builtin(num_workgroups) num_w: vec3<u32>,
    ) {
    grid[global_id.x] = grid[global_id.x] + 5.;
}

