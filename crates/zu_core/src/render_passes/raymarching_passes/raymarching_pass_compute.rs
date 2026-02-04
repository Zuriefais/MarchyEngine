use bytemuck::{Pod, Zeroable, bytes_of};
use egui_probe::EguiProbe;
use wgpu::{
    CommandEncoder, ComputePipelineDescriptor, Device, PushConstantRange, ShaderStages,
    util::RenderEncoder,
};

use crate::texture_manager::{TextureManager, textures::EngineTexture};

#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy, Zeroable, Pod)]
struct RaymarchingConstants {}

pub struct RaymarchingRenderComputePass {
    compute_pipeline: wgpu::ComputePipeline,
}

impl RaymarchingRenderComputePass {
    pub fn new(device: &Device, texture_manager: &mut TextureManager) -> Self {
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/raymarching_compute.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raymarching compute pass layout descriptor"),
            bind_group_layouts: &[texture_manager.get_compute_mut_bind_group_layout()],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Raymarching compute pass"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("compute_main"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });
        RaymarchingRenderComputePass { compute_pipeline }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        texture_manager: &TextureManager,
        width: u32,
        height: u32,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Raymarching compute pass"),
            timestamp_writes: Default::default(),
        });
        compute_pass.set_pipeline(&self.compute_pipeline);

        compute_pass.set_bind_group(
            0,
            texture_manager
                .get_texture("Raymarching")
                .unwrap()
                .compute_mut_group_f32(),
            &[],
        );
        let wg_x = (width + 7) / 16;
        let wg_y = (height + 7) / 16;
        compute_pass.dispatch_workgroups(wg_x, wg_y, 1);
    }
}

#[derive(Debug, Clone, Copy, EguiProbe)]
pub struct RaymarchingOptions {}

impl Default for RaymarchingOptions {
    fn default() -> Self {
        Self {}
    }
}
