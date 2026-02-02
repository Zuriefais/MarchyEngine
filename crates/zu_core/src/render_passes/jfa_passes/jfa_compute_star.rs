use bytemuck::{Pod, Zeroable, bytes_of};
use rand::RngCore;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipelineDescriptor, Device,
    PushConstantRange, ShaderStages,
    util::{DeviceExt, RenderEncoder},
};

use crate::texture_manager::{TextureManager, textures::EngineTexture};

fn create_noise_buffer(device: &Device, width: u32, height: u32) -> Buffer {
    let noise_data: Vec<f32> = (0..width * height)
        .map(|_| rand::rng().next_u32() as f32)
        .collect();
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Noise Buffer"),
        contents: bytemuck::cast_slice(&noise_data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Zeroable, Pod)]
pub struct JfaConstants {
    pub one_over_size: [f32; 2],
    pub texture_size: [f32; 2],
    pub u_offset: f32, // <-- new: jump distance for this pass
    pub _pad: f32,     // keep alignment (32 bytes total)
}

pub struct JfaComputeStarPass {
    compute_pipeline: wgpu::ComputePipeline,
    noise_bind_group_layout: BindGroupLayout,
    noise_bind_group: BindGroup,
}

impl JfaComputeStarPass {
    pub fn new(
        device: &Device,
        texture_manager: &mut TextureManager,
        width: u32,
        height: u32,
    ) -> Self {
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/jfa_compute_star.wgsl"));
        let noise_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Noise Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Jfa compute pass layout descriptor"),
            bind_group_layouts: &[
                texture_manager.get_compute_bind_group_layout(),
                texture_manager.get_compute_mut_bind_group_layout(),
                &noise_bind_group_layout,
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

        let noise_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Noise Bind Group"),
            layout: &noise_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: create_noise_buffer(device, width, height).as_entire_binding(),
            }],
        });

        JfaComputeStarPass {
            compute_pipeline,
            noise_bind_group_layout,
            noise_bind_group,
        }
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        self.noise_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Noise Bind Group"),
            layout: &self.noise_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: create_noise_buffer(device, width, height).as_entire_binding(),
            }],
        })
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
        let wg_x = (width + 7) / 16;
        let wg_y = (height + 7) / 16;
        compute_pass.set_pipeline(&self.compute_pipeline);
        let p = width.max(height) as f32;
        for pass_i in 0..passes {
            let u_offset = p / 3f32.powi(pass_i as i32);

            compute_pass.set_push_constants(
                0,
                bytes_of(&JfaConstants {
                    one_over_size: [1.0 / width as f32, 1.0 / height as f32],
                    texture_size: [width as f32, height as f32],
                    u_offset,
                    _pad: 0.0,
                }),
            );

            let (src, dst) = if pass_i % 2 == 0 {
                (
                    texture_manager.get_texture("JfaTexture").unwrap(),
                    texture_manager.get_texture("JfaTexture1").unwrap(),
                )
            } else {
                (
                    texture_manager.get_texture("JfaTexture1").unwrap(),
                    texture_manager.get_texture("JfaTexture").unwrap(),
                )
            };

            compute_pass.set_bind_group(0, src.compute_bind_group(), &[]);
            compute_pass.set_bind_group(1, dst.compute_mut_group_f32(), &[]);
            compute_pass.set_bind_group(2, &self.noise_bind_group, &[]);
            compute_pass.dispatch_workgroups(wg_x, wg_y, 1);
        }
    }
}
