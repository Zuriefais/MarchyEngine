struct FragmentInput {
    @location(0) frag_color: vec4<f32>,
};

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    return in.frag_color;
}

struct Camera {
    proj_mat: mat4x4<f32>,
    cam_pos: vec2<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
    @location(0) vert_position: vec2<f32>,
    @location(1) position: vec2<f32>,
    @location(2) scale: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = (in.vert_position * in.scale) + in.position - camera.cam_pos;
    out.position = vec4<f32>(world_pos, 0.0, 1.0) * camera.proj_mat;
    out.frag_color = in.color;
    return out;
}
