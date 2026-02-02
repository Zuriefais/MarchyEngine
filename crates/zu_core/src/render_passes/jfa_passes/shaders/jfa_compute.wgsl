@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(1) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;

struct PushConstants {
    one_over_size: vec2<f32>,
    texture_size: vec2<f32>,
    u_offset: i32,
    _pad: f32,
};

var<push_constant> constants: PushConstants;

@compute @workgroup_size(32, 32)
fn fs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixelCoord = id.xy;

    let current_value = textureLoad(input_texture, vec2<i32>(pixelCoord), 0);
    if current_value.z == 1.0 {
        textureStore(output_texture, vec2<i32>(pixelCoord), current_value);
        return;
    }

    let uv = (vec2<f32>(pixelCoord) + 0.5) * constants.one_over_size;

    var nearestSeed = vec4(-2.0);
    var nearestDist = 999999.9;

    let textureDim = textureDimensions(input_texture) - 1;

    for (var y: i32 = -1; y <= 1; y += 1) {
        for (var x: i32 = -1; x <= 1; x += 1) {
            let offset = vec2(x, y) * constants.u_offset;
            let sampleCoord = vec2<i32>(pixelCoord) + offset;

            let clampedCoord = clamp(sampleCoord, vec2<i32>(0), vec2<i32>(textureDim));

            let sample_value = textureLoad(input_texture, clampedCoord, 0);
            let sample_seed = sample_value.xy;

            if sample_seed.x != 0.0 || sample_seed.y != 0.0 {
                let diff = sample_seed - uv;
                let dist = dot(diff, diff);
                if dist < nearestDist {
                    nearestDist = dist;
                    nearestSeed = sample_value;
                }
            }
        }
    }

    textureStore(output_texture, vec2<i32>(pixelCoord), nearestSeed);
}
