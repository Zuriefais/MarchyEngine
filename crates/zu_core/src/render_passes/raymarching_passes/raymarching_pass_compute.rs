use std::{ops::Range, time::SystemTime};

use bytemuck::{Pod, Zeroable, bytes_of};
use egui_probe::EguiProbe;
use wgpu::{
    CommandEncoder, ComputePipelineDescriptor, Device, PushConstantRange, ShaderStages,
    util::RenderEncoder,
};

use crate::texture_manager::{TextureManager, textures::EngineTexture};

#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy, Zeroable, Pod)]
struct RaymarchingConstants {
    texture_size: [f32; 2],
    time: f32,
    rotation: f32,
    ray_origin: Vec3,
    FOV: f32,
}

pub struct RaymarchingRenderComputePass {
    compute_pipeline: wgpu::ComputePipeline,
    current_time: SystemTime,
}

impl RaymarchingRenderComputePass {
    pub fn new(device: &Device, texture_manager: &mut TextureManager) -> Self {
        use std::fs;

        let source = fs::read_to_string(
            "crates/zu_core/src/render_passes/raymarching_passes/shaders/raymarching_compute.wgsl",
        )
        .expect("Failed to read raymarching_compute.wgsl");
        include_bytes!()
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raymarching compute shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raymarching compute pass layout descriptor"),
            bind_group_layouts: &[texture_manager.get_compute_mut_bind_group_layout()],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<RaymarchingConstants>() as u32,
            }],
        });

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Raymarching compute pass"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("compute_main"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });
        RaymarchingRenderComputePass {
            compute_pipeline,
            current_time: SystemTime::now(),
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        texture_manager: &TextureManager,
        width: u32,
        height: u32,
        FOV: f32,
        rotation: f32,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Raymarching compute pass"),
            timestamp_writes: Default::default(),
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_push_constants(
            0,
            bytes_of(&RaymarchingConstants {
                texture_size: [width as f32, height as f32],
                time: self.current_time.elapsed().unwrap().as_secs_f32(),
                rotation: rotation,
                FOV,
            }),
        );
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
