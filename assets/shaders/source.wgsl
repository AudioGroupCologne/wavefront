@group(0) @binding(0)
var<storage, read_write> grid: array<f32>;

//TODO: make this an array of structs
@group(0) @binding(1)
var<storage, read_write> sources: array<Source>;

struct Source {
    index: f32,
    value: f32,
}

@compute @workgroup_size(1)
fn main() {
    // SOURCES
    for (var i: u32 = 0u; i < arrayLength(&sources); i++) {
        let index = u32(sources[i].index);
        grid[index + 4u] = sources[i].value;
        grid[index + 5u] = sources[i].value;
        grid[index + 6u] = sources[i].value;
        grid[index + 7u] = sources[i].value;
    }
}