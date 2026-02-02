@group(0) @binding(0) var my_sampler: sampler;
@group(0) @binding(1) var input_texture: texture_2d<f32>;

struct PushConstants {
    one_over_size: vec2<f32>,
    u_offset: f32
}

var<push_constant> constants: PushConstants;


struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var nearestSeed = vec4(-2.0);
    var nearestDist = 999999.9;

    for (var y = -1.0; y <= 1.0; y += 1.0) {
        for (var x = -1.0; x <= 1.0; x += 1.0) {
            var sample_uv = input.uv + vec2(x, y) * constants.u_offset * constants.one_over_size;

            if sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0 { continue; }

            var sample_value = textureSample(input_texture, my_sampler, sample_uv);
            var sample_seed = sample_value.xy;

            if sample_seed.x != 0.0 || sample_seed.y != 0.0 {
                var diff = sample_seed - input.uv;
                var dist = dot(diff, diff);
                if dist < nearestDist {
                    nearestDist = dist;
                    nearestSeed = sample_value;
                }
            }
        }
    }
    return nearestSeed;
}
