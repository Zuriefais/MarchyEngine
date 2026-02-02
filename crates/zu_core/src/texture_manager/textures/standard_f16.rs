use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Device, Sampler,
    Texture, TextureView,
};

use crate::texture_manager::{
    BindGroupLayouts,
    textures::EngineTexture,
};

pub struct StandardTextureF16 {
    pub texture: Texture,
    pub view: TextureView,
    pub bind_group: BindGroup,
    pub compute_bind_group: BindGroup,
    pub compute_mut_bind_group: BindGroup,
    pub resolution_scale: f32,
}

impl StandardTextureF16 {
    pub fn new(
        name: &str,
        resolution: (u32, u32),
        device: &Device,
        bind_group_layouts: &BindGroupLayouts,
        sampler: &Sampler,
        resolution_scale: f32,
    ) -> Self {
        let texture_size = wgpu::Extent3d {
            width: (resolution.0 as f32 * resolution_scale) as u32,
            height: (resolution.1 as f32 * resolution_scale) as u32,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::STORAGE_BINDING,
            label: Some(name),
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &bind_group_layouts.texture,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
        });
        let compute_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Compute texture Bind Group"),
            layout: &bind_group_layouts.compute_texture,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });
        let compute_mut_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Compute mut texture Bind Group"),
            layout: &bind_group_layouts.compute_mut_texture,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });
        Self {
            view,
            bind_group,
            compute_bind_group,
            compute_mut_bind_group,
            texture,
            resolution_scale,
        }
    }
}

impl EngineTexture for StandardTextureF16 {
    fn view(&self) -> &TextureView {
        &self.view
    }

    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn compute_bind_group(&self) -> &BindGroup {
        &self.compute_bind_group
    }

    fn compute_mut_group_f16(&self) -> Option<&BindGroup> {
        Some(&self.compute_mut_bind_group)
    }

    fn compute_mut_group_f32(&self) -> Option<&BindGroup> {
        None
    }

    fn resize(
        &mut self,
        resolution: (u32, u32),
        device: &Device,
        bind_group_layouts: &BindGroupLayouts,
        sampler: &Sampler,
        resolution_scale: f32,
        name: &str,
    ) {
        *self = Self::new(
            name,
            resolution,
            device,
            bind_group_layouts,
            sampler,
            resolution_scale,
        )
    }

    fn resolution_scale(&self) -> f32 {
        self.resolution_scale
    }
}
