use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use wgpu::{Buffer, Device, RenderPass, ShaderModule, util::DeviceExt};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct QuadVertex {
    position: Vec2,
}

impl QuadVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex, // per-vertex
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0, // matches @location(0)
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

// a unit quad (two triangles)
pub const QUAD_VERTICES: &[QuadVertex] = &[
    QuadVertex {
        position: Vec2::new(-0.5, -0.5),
    },
    QuadVertex {
        position: Vec2::new(0.5, -0.5),
    },
    QuadVertex {
        position: Vec2::new(0.5, 0.5),
    },
    QuadVertex {
        position: Vec2::new(-0.5, 0.5),
    },
];

pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub struct QuadVertexRenderPass {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    pub shader: ShaderModule,
}

impl QuadVertexRenderPass {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/quad_vertex.wgsl"));

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            shader,
        }
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..(QUAD_INDICES.len() as u32), 0, 0..1);
    }
}

#[macro_export]
macro_rules! vertex_state_for_quad {
    ($render_pass:expr) => {
        wgpu::VertexState {
            module: &$render_pass.shader,
            entry_point: Some("vs_main"),
            buffers: &[$crate::render_passes::quad_vertex::QuadVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }
    };
}
