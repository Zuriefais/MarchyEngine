@group(0) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;

@compute @workgroup_size(16, 16)
fn compute_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixelCoord = id.xy;
    textureStore(output_texture, vec2<i32>(pixelCoord), vec4(f32(id.x*100), f32(id.y*100), 1.0, 1.0));
}
