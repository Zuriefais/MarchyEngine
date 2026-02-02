use bytemuck::{Pod, Zeroable, bytes_of};
use glam::Vec2;
use wgpu::{
    BindGroup, CommandEncoder, Device, PushConstantRange,
    ShaderStages, TextureView,
    util::RenderEncoder,
};

use crate::{
    render_passes::quad_vertex::QuadVertexRenderPass,
    texture_manager::{TextureManager, textures::EngineTexture},
    vertex_state_for_quad,
};

#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy, Zeroable, Pod)]
struct JfaConstants {
    one_over_size: Vec2,
    u_offset: f32,
}

pub struct JfaRenderPass {
    render_pipeline: wgpu::RenderPipeline,
    texture1: usize,
    texture2: usize,
}

impl JfaRenderPass {
    pub fn new(
        device: &Device,
        quad_render_pass: &QuadVertexRenderPass,
        texture_manager: &mut TextureManager,
        texture1: usize,
        texture2: usize,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/jfa.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("JFA Pipeline Layout"),
            bind_group_layouts: &[texture_manager.get_bind_group_layout()],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::FRAGMENT,
                range: 0..std::mem::size_of::<JfaConstants>() as u32,
            }],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("JFA Render Pipeline"),
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

        JfaRenderPass {
            render_pipeline,
            texture1,
            texture2,
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        input_texture_bind_group: &BindGroup,
        output_view: &TextureView,
        offset: f32,
        quad_render_pass: &QuadVertexRenderPass,
        width: u32,
        height: u32,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("JFA Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
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
        render_pass.set_push_constants(
            ShaderStages::FRAGMENT,
            0,
            bytes_of(&JfaConstants {
                one_over_size: Vec2::new(1.0 / width as f32, 1.0 / height as f32),
                u_offset: offset,
            }),
        );
        render_pass.set_bind_group(0, input_texture_bind_group, &[]);
        quad_render_pass.render(&mut render_pass);
    }

    pub fn multi_render(
        &mut self,
        encoder: &mut CommandEncoder,
        quad_render_pass: &QuadVertexRenderPass,
        texture_manager: &TextureManager,
        passes: u32,
        width: u32,
        height: u32,
    ) {
        let (texture1, texture2) = (
            texture_manager.get_texture_by_index(self.texture1).unwrap(),
            texture_manager.get_texture_by_index(self.texture2).unwrap(),
        );
        self.render(
            encoder,
            texture_manager
                .get_texture("SceneTexture")
                .unwrap()
                .bind_group(),
            texture2.view(),
            2.0f32.powi((passes - 1) as i32),
            quad_render_pass,
            width,
            height,
        );
        for i in 0..passes {
            let (texture1, texture2) = if i % 2 == 0 {
                (texture1, texture2)
            } else {
                (texture2, texture1)
            };

            self.render(
                encoder,
                texture1.bind_group(),
                texture2.view(),
                2.0f32.powi((passes - i - 1) as i32),
                quad_render_pass,
                width,
                height,
            );
        }
    }
}
