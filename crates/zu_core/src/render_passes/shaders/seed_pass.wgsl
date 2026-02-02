@group(0) @binding(0) var my_sampler: sampler;
@group(0) @binding(1) var input_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var alpha = textureSample(input_texture, my_sampler, input.uv).a;
    if alpha > 0.0 {return vec4(input.uv * alpha, 0.0, 1.0);} else {return vec4(0.0, 0.0, 0.0, 0.0);}
}
