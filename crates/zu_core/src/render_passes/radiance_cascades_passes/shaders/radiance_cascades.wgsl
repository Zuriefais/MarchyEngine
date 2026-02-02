@group(0) @binding(0) var scene_sampler: sampler;
@group(0) @binding(1) var scene_texture: texture_2d<f32>;

@group(1) @binding(0) var distance_sampler: sampler;
@group(1) @binding(1) var distance_texture: texture_2d<f32>;

@group(2) @binding(0) var prev_sampler: sampler;
@group(2) @binding(1) var prev_texture: texture_2d<f32>;


struct PushConstants {
    ray_count: i32,
    accum_radiance: i32,
    max_steps: i32,
    enable_noise: i32,
    show_grain: i32,
    resolution: vec2<f32>
}

var<push_constant> constants: PushConstants;

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

const PI: f32 = 3.14159265;
const TAU: f32 = 2.0 * PI;
const EPS = 0.001f;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light = textureSample(scene_texture, scene_sampler, input.uv);
    let uv = input.uv;
    if light.a > 0.1 {
        return light;
    }
    var radiance = vec4(0.0);

    var one_over_ray_count = 1.0 / f32(constants.ray_count);
    var angleStepSize = TAU * one_over_ray_count;

    var offset = select(0.0, rand22(uv), constants.enable_noise != 0);
    var rayAngleStepSize = select(angleStepSize + offset * TAU, angleStepSize, constants.show_grain != 0);

    for (var i = 0; i < constants.ray_count; i++) {
        var angle = rayAngleStepSize * (f32(i) + offset);
        var rayDirection = vec2(cos(angle), -sin(angle));

        var sample_uv = uv;
        var radDelta = vec4(0.0);
        var hitSurface = false;

        for (var step = 1; step < constants.max_steps; step++) {

            var dist = textureSample(distance_texture, distance_sampler, sample_uv).r;

            sample_uv += rayDirection * dist;

            if outOfBounds(sample_uv) {break;}

            if dist < EPS {
                var sampleColor = textureSample(scene_texture, scene_sampler, sample_uv);
                radDelta += sampleColor;
                hitSurface = true;
                break;
            }
        }
        radiance += radDelta;
    }
    return vec4((radiance * one_over_ray_count).xyz, 1.0);
}

fn outOfBounds(uv: vec2<f32>) -> bool {
    return uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0;
}

fn rand22(n: vec2f) -> f32 { return fract(sin(dot(n, vec2f(12.9898, 4.1414))) * 43758.5453); }
