struct VertexInput {
    @location(0) position: vec2<f32>, // from QuadVertex
};

// Vertex output / fragment input
struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Map vertex positions to clip space (-1..1)
    out.clip_pos = vec4<f32>(input.position * 2.0, 0.0, 1.0);

    // Map quad positions (-0.5..0.5) to UVs (0..1)
    out.uv = vec2(
        input.position.x + 0.5,
        1.0 - (input.position.y + 0.5)
    );

    return out;
}
