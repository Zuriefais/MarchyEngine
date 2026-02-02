use bytemuck::{Pod, Zeroable, bytes_of};
use wgpu::{
    CommandEncoder, ComputePipelineDescriptor, Device,
    PushConstantRange, ShaderStages,
    util::RenderEncoder,
};

use crate::texture_manager::{TextureManager, textures::EngineTexture};

#[repr(C)]
#[derive(Debug, Copy, Clone, Zeroable, Pod)]
pub struct JfaConstants {
    pub one_over_size: [f32; 2],
    pub texture_size: [f32; 2],
    pub passes: i32,
    pub _pad: i32,
}

pub struct JfaComputeOneShotPass {
    compute_pipeline: wgpu::ComputePipeline,
}

impl JfaComputeOneShotPass {
    pub fn new(device: &Device, texture_manager: &mut TextureManager) -> Self {
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/jfa_compute_one_shot.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Jfa compute pass layout descriptor"),
            bind_group_layouts: &[
                texture_manager.get_compute_bind_group_layout(),
                texture_manager.get_compute_mut_bind_group_layout(),
            ],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<JfaConstants>() as u32,
            }],
        });

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Jfa compute pass"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        JfaComputeOneShotPass { compute_pipeline }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        texture_manager: &TextureManager,
        passes: u32,
        width: u32,
        height: u32,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("JFA compute pass"),
            timestamp_writes: Default::default(),
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_push_constants(
            0,
            bytes_of(&JfaConstants {
                one_over_size: [1.0 / width as f32, 1.0 / height as f32],
                texture_size: [width as f32, height as f32],
                passes: passes as i32,
                _pad: 0,
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
                .get_texture("JfaTexture")
                .unwrap()
                .compute_mut_group_f32(),
            &[],
        );
        let wg_x = width.div_ceil(8);
        let wg_y = height.div_ceil(8);
        compute_pass.dispatch_workgroups(wg_x, wg_y, 1);
    }
}
