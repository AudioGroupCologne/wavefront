@group(0) @binding(0) var texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(1) var<storage, read_write> grid: array<f32>;

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let color = vec4<f32>(1., 1., 1., 1.);

    storageBarrier();

    textureStore(texture, location, color);
}