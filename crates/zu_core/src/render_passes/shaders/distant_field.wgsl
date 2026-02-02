@group(0) @binding(0) var my_sampler: sampler;
@group(0) @binding(1) var jfa_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var nearestSeed: vec2<f32> = textureSample(jfa_texture, my_sampler, input.uv).xy;
    var distance: f32 = clamp(distance(input.uv, nearestSeed), 0.0, 1.0);
    return vec4(vec3(distance), 1.0);
}
