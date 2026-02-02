@group(0) @binding(0) var my_sampler: sampler;
@group(0) @binding(1) var input_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(input_texture, my_sampler, input.uv);
}
