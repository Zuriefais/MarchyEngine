@group(0) @binding(0) var my_sampler: sampler;
@group(0) @binding(1) var my_texture: texture_2d<f32>;

struct PushConstants {
    ray_count: i32,
    size: vec2<f32>,
    accum_radiance: i32,
    max_steps: i32,
    enable_noise: i32
}

var<push_constant> constants: PushConstants;

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

const PI: f32 = 3.14159265;
const TAU: f32 = 2.0 * PI;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(my_texture, my_sampler, input.uv);
    return vec4(raymarch(color, input.uv).xyz, 1.0);
}

fn outOfBounds(uv: vec2<f32>) -> bool {
    return uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0;
}

fn rand22(n: vec2f) -> f32 { return fract(sin(dot(n, vec2f(12.9898, 4.1414))) * 43758.5453); }

fn raymarch(light: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    if light.a > 0.1 {
        return light;
    }
    var one_over_ray_count: f32 = 1.0 / f32(constants.ray_count);
    var tau_over_ray_count: f32 = TAU * one_over_ray_count;

    var noise: f32 = select(0.1, rand22(uv), bool(constants.enable_noise));
    var radiance = vec4(0.0);

    for (var i = 0; i < constants.ray_count; i++) {
        var angle = tau_over_ray_count * (f32(i) + noise);
        var rayDirectionUv = vec2(cos(angle), -sin(angle)) / constants.size;
        var traveled = vec2(0.0);

        let initial_step: i32 = select(max(0, constants.max_steps - 1), 0, bool(constants.accum_radiance));
        for (var step = initial_step; step < constants.max_steps; step++) {
            var sampleUv = uv + rayDirectionUv * f32(step);

            if sampleUv.x < 0.0 || sampleUv.x > 1.0 || sampleUv.y < 0.0 || sampleUv.y > 1.0 {
                break;
            }

            var sampleLight = textureSample(my_texture, my_sampler, sampleUv);
            if sampleLight.a > 0.5 {
                radiance += sampleLight;
                break;
            }
        }
    }

    return radiance * one_over_ray_count;
}
