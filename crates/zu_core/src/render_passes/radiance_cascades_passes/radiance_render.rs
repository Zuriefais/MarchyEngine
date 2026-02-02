use bytemuck::{Pod, Zeroable, bytes_of};
use egui_probe::EguiProbe;
use wgpu::{
    CommandEncoder, Device, PushConstantRange, ShaderStages,
    util::RenderEncoder,
};

use crate::{
    render_passes::quad_vertex::QuadVertexRenderPass,
    texture_manager::{TextureManager, textures::EngineTexture},
    vertex_state_for_quad,
};

#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy, Zeroable, Pod)]
struct RadianceCascadesConstants {
    ray_count: i32,
    accum_radiance: i32,
    max_steps: i32,
    enable_noise: i32,
    show_grain: i32,
}

pub struct RadianceRenderPass {
    render_pipeline: wgpu::RenderPipeline,
}

impl RadianceRenderPass {
    pub fn new(
        device: &Device,
        quad_render_pass: &QuadVertexRenderPass,
        texture_manager: &mut TextureManager,
    ) -> Self {
        let radiance_shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/radiance_cascades.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    texture_manager.get_bind_group_layout(),
                    texture_manager.get_bind_group_layout(),
                ],
                push_constant_ranges: &[PushConstantRange {
                    stages: ShaderStages::FRAGMENT,
                    range: 0..std::mem::size_of::<RadianceCascadesConstants>() as u32,
                }],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: vertex_state_for_quad!(quad_render_pass),
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &radiance_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: wgpu::TextureFormat::Rgba32Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,     // 6.
        });

        RadianceRenderPass { render_pipeline }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        options: RadiansOptions,
        texture_manager: &TextureManager,
        quad_render_pass: &QuadVertexRenderPass,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Radiance render pass"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: texture_manager
                        .get_texture("RadianceCascades")
                        .unwrap()
                        .view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: Default::default(),
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: Default::default(),
            occlusion_query_set: Default::default(),
        });

        render_pass.set_pipeline(&self.render_pipeline); // 2.
        render_pass.set_push_constants(
            ShaderStages::FRAGMENT,
            0,
            bytes_of(&RadianceCascadesConstants {
                ray_count: options.ray_count as i32,
                accum_radiance: options.accum_radiance as i32,
                max_steps: options.max_steps as i32,
                enable_noise: options.enable_noise as i32,
                show_grain: options.show_grain as i32,
            }),
        );
        render_pass.set_bind_group(
            0,
            texture_manager
                .get_texture("SceneTexture")
                .unwrap()
                .bind_group(),
            &[],
        );
        render_pass.set_bind_group(
            1,
            texture_manager
                .get_texture("DistanceField")
                .unwrap()
                .bind_group(),
            &[],
        );
        quad_render_pass.render(&mut render_pass);
    }
}

#[derive(Debug, Clone, Copy, EguiProbe)]
pub struct RadiansOptions {
    ray_count: u32,
    accum_radiance: bool,
    max_steps: u32,
    enable_noise: bool,
    show_grain: bool,
}

impl Default for RadiansOptions {
    fn default() -> Self {
        Self {
            ray_count: 8,
            accum_radiance: true,
            max_steps: 128,
            enable_noise: true,
            show_grain: true,
        }
    }
}
