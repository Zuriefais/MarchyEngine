use std::{num::NonZero, ops::Range, time::SystemTime};

use bytemuck::{NoUninit, Pod, Zeroable, bytes_of, cast_slice};
use egui::{Response, Ui};
use egui_probe::{EguiProbe, Style};
use glam::Vec3;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferDescriptor, BufferUsages, CommandEncoder, ComputePipelineDescriptor,
    Device, DynamicOffset, PushConstantRange, Queue, ShaderStages, util::RenderEncoder,
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
    objects_count: u32,
}

#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy, Zeroable, Pod, EguiProbe)]
pub struct RaymarchingObject {
    #[egui_probe(with probe_vec3)]
    pub position: Vec3,
    pub radius: f32,
}

fn probe_vec3(value: &mut Vec3, ui: &mut Ui, _style: &Style) -> Response {
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut value.x).speed(0.01));
        ui.add(egui::DragValue::new(&mut value.y).speed(0.01));
        ui.add(egui::DragValue::new(&mut value.z).speed(0.01));
    })
    .response
}

impl Default for RaymarchingObject {
    fn default() -> Self {
        Self {
            position: Default::default(),
            radius: 0.5,
        }
    }
}

pub struct RaymarchingRenderComputePass {
    compute_pipeline: wgpu::ComputePipeline,
    current_time: SystemTime,
    storage_bind_group: BindGroup,
    storage_buffer: Buffer,
    objects_count: usize,
}

impl RaymarchingRenderComputePass {
    pub fn new(device: &Device, queue: &Queue, texture_manager: &mut TextureManager) -> Self {
        use std::fs;

        let source = fs::read_to_string(
            "crates/zu_core/src/render_passes/raymarching_passes/shaders/raymarching_compute.wgsl",
        )
        .expect("Failed to read raymarching_compute.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raymarching compute shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        let storage_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Raymarching objects bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: NonZero::new(size_of::<RaymarchingObject>() as u64),
                    },
                    count: None,
                }],
            });

        let storage_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Raymarching objects"),
            size: (size_of::<RaymarchingObject>() * 256) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(
            &storage_buffer,
            0,
            bytes_of(&[RaymarchingObject {
                position: Vec3::ZERO,
                radius: 0.5,
            }]),
        );

        let storage_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Raymarching objects bind group"),
            layout: &storage_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(storage_buffer.as_entire_buffer_binding()),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raymarching compute pass layout descriptor"),
            bind_group_layouts: &[
                texture_manager.get_compute_mut_bind_group_layout(),
                &storage_bind_group_layout,
            ],
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
            storage_bind_group,
            objects_count: 1,
            storage_buffer,
        }
    }

    pub fn render(
        &mut self,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        texture_manager: &TextureManager,
        width: u32,
        height: u32,
        FOV: f32,
        rotation: f32,
        ray_origin: Vec3,
        objects: &[RaymarchingObject],
    ) {
        let required_size = (size_of::<RaymarchingObject>() * objects.len()) as u64;
        if required_size < self.storage_buffer.size() {
            queue.write_buffer(&self.storage_buffer, 0, cast_slice(objects));
        } else {
            panic!("Buffer expansion logic")
        }

        self.objects_count = objects.len();

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
                ray_origin,
                rotation: rotation,
                FOV,
                objects_count: objects.len() as u32,
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
        compute_pass.set_bind_group(1, Some(&self.storage_bind_group), &[]);
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
