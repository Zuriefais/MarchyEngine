use bytemuck::{Pod, Zeroable, bytes_of};
use egui_probe::EguiProbe;
use wgpu::{
    CommandEncoder, ComputePipelineDescriptor, Device,
    PushConstantRange, ShaderStages,
    util::RenderEncoder,
};

use crate::texture_manager::{TextureManager, textures::EngineTexture};

#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy, Zeroable, Pod)]
struct RadianceCascadesConstants {
    pub ray_count: i32,
    pub accum_radiance: i32,
    pub max_steps: i32,
    pub enable_noise: i32,
    pub show_grain: i32,
    pub _padding0: i32,
    pub resolution: [f32; 2],
    pub _padding1: [f32; 2], // чтобы размер был кратен 16 байт
}

pub struct RadianceRenderComputePass {
    compute_pipeline: wgpu::ComputePipeline,
}

impl RadianceRenderComputePass {
    pub fn new(device: &Device, texture_manager: &mut TextureManager) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!(
            "./shaders/radiance_cascades_compute.wgsl"
        ));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Radiance compute pass layout descriptor"),
            bind_group_layouts: &[
                texture_manager.get_compute_bind_group_layout(),
                texture_manager.get_compute_bind_group_layout(),
                texture_manager.get_compute_mut_bind_group_layout(),
            ],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<RadianceCascadesConstants>() as u32,
            }],
        });

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Radiance compute pass"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });
        RadianceRenderComputePass { compute_pipeline }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        texture_manager: &TextureManager,
        options: RadiansOptions,
        width: u32,
        height: u32,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Radiance compute pass"),
            timestamp_writes: Default::default(),
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_push_constants(
            0,
            bytes_of(&RadianceCascadesConstants {
                ray_count: options.ray_count as i32,
                accum_radiance: options.accum_radiance as i32,
                max_steps: options.max_steps as i32,
                enable_noise: options.enable_noise as i32,
                show_grain: options.show_grain as i32,
                _padding0: 0,
                resolution: [width as f32, height as f32],
                _padding1: [0.0, 0.0],
            }),
        );
        compute_pass.set_bind_group(
            0,
            texture_manager
                .get_texture("SceneTexture")
                .unwrap()
                .compute_bind_group(),
            &[],
        );
        compute_pass.set_bind_group(
            1,
            texture_manager
                .get_texture("DistanceField")
                .unwrap()
                .compute_bind_group(),
            &[],
        );
        compute_pass.set_bind_group(
            2,
            texture_manager
                .get_texture("RadianceCascades")
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
