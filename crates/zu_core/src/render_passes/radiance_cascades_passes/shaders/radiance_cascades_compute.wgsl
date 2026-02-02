@group(0) @binding(0) var scene_texture: texture_2d<f32>;

@group(1) @binding(0) var distance_texture: texture_2d<f32>;

@group(2) @binding(0) var output_texture: texture_storage_2d<rgba32float, read_write>;


struct PushConstants {
    ray_count: i32,
    accum_radiance: i32,
    max_steps: i32,
    enable_noise: i32,
    show_grain: i32,
    _padding0: i32,
    resolution: vec2<f32>,
    _padding1: vec2<f32>
};



var<push_constant> constants: PushConstants;

const PI: f32 = 3.14159265;
const TAU: f32 = 2.0 * PI;
const EPS = 0.001f;

@compute @workgroup_size(16, 16)
fn fs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixelCoord = id.xy;
    let uv = (vec2<f32>(pixelCoord) + 0.5) / constants.resolution;

    let light = textureLoad(scene_texture, pixelCoord, 0);
    if light.a > 0.1 {
        textureStore(output_texture, vec2<i32>(pixelCoord), light);
        return;
    }
    var radiance = vec4(0.0);

    var one_over_ray_count = 1.0 / f32(constants.ray_count);
    var angleStepSize = TAU * one_over_ray_count;

    var offset = select(0.0, rand22(uv), constants.enable_noise != 0);
    var rayAngleStepSize = select(angleStepSize + offset * TAU, angleStepSize, constants.show_grain != 0);
    let pixelSize = 1.0 / constants.resolution;

    for (var i = 0; i < constants.ray_count; i++) {
        var angle = rayAngleStepSize * (f32(i) + offset);
        var rayDirection = vec2(cos(angle), -sin(angle));

        var sample_uv = uv;
        var radDelta = vec4(0.0);
        var hitSurface = false;
        for (var step = 1; step < constants.max_steps; step++) {

            var sample_px = vec2<i32>(sample_uv * constants.resolution);
            var dist = textureLoad(distance_texture, sample_px, 0).r;

            sample_uv += rayDirection * dist;

            if outOfBounds(sample_uv) {break;}

            if dist < EPS {
                var sampleColor = textureLoad(scene_texture, sample_px, 0);
                radDelta += sampleColor;
                hitSurface = true;
                break;
            }
        }
        radiance += radDelta;
    }
    textureStore(output_texture, vec2<i32>(pixelCoord), vec4((radiance * one_over_ray_count).xyz, 1.0));
}

fn outOfBounds(uv: vec2<f32>) -> bool {
    return uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0;
}

fn rand22(n: vec2f) -> f32 { return fract(sin(dot(n, vec2f(12.9898, 4.1414))) * 43758.5453); }
