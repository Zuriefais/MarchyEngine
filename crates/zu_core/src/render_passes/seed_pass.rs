use wgpu::{
    CommandEncoder, Device,
};

use crate::{render_passes::quad_vertex::QuadVertexRenderPass, texture_manager::TextureManager};
use crate::{texture_manager::textures::EngineTexture, vertex_state_for_quad};

pub struct SeedRenderPass {
    render_pipeline: wgpu::RenderPipeline,
}

impl SeedRenderPass {
    pub fn new(
        device: &Device,
        texture_manager: &TextureManager,
        quad_render_pass: &QuadVertexRenderPass,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/seed_pass.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Seed Pipeline Layout"),
            bind_group_layouts: &[texture_manager.get_bind_group_layout()],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Seed Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: vertex_state_for_quad!(quad_render_pass),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba32Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        SeedRenderPass { render_pipeline }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        texture_manager: &TextureManager,
        quad_render_pass: &QuadVertexRenderPass,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Seed  Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture_manager.get_texture("JfaTexture").unwrap().view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: Default::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: Default::default(),
            occlusion_query_set: Default::default(),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(
            0,
            texture_manager
                .get_texture("SceneTexture")
                .unwrap()
                .bind_group(),
            &[],
        );
        quad_render_pass.render(&mut render_pass);
    }
}
