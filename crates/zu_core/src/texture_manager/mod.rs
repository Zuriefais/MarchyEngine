pub mod textures;

use indexmap::IndexMap;
use wgpu::{BindGroupLayout, Device, Sampler, TextureFormat};

use crate::texture_manager::textures::{EngineTexture, ManagedTexture, TextureType};

pub struct BindGroupLayouts {
    compute_texture: BindGroupLayout,
    compute_mut_texture: BindGroupLayout,
    texture: BindGroupLayout,
}

impl BindGroupLayouts {
    pub fn new(device: &Device) -> Self {
        Self {
            texture: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            }),
            compute_texture: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute texture Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            }),
            compute_mut_texture: device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Compute mut texture Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                },
            ),
        }
    }
}

pub struct TextureManager {
    textures: IndexMap<String, ManagedTexture>,
    bind_group_layouts: BindGroupLayouts,
    sampler: Sampler,
    num: usize,
}

impl TextureManager {
    pub fn new(device: &Device) -> Self {
        Self {
            textures: Default::default(),
            bind_group_layouts: BindGroupLayouts::new(device),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            num: 0,
        }
    }

    pub fn create_texture(
        &mut self,
        name: &str,
        resolution: (u32, u32),
        device: &Device,
        texture_type: TextureType,
        resolution_scale: f32,
    ) -> usize {
        self.textures.insert(
            name.to_string(),
            ManagedTexture::new(
                name,
                resolution,
                device,
                &self.bind_group_layouts,
                &self.sampler,
                resolution_scale,
                texture_type,
            ),
        );
        self.num += 1;
        self.num - 1
    }

    pub fn get_texture(&self, name: &str) -> Option<&ManagedTexture> {
        self.textures.get(name)
    }

    pub fn get_texture_by_index(&self, i: usize) -> Option<&ManagedTexture> {
        self.textures.get_index(i).map(|(_, v)| v)
    }

    pub fn get_texture_mut(&mut self, name: &str) -> Option<&mut ManagedTexture> {
        self.textures.get_mut(name)
    }

    pub fn get_texture_by_index_mut(&mut self, i: usize) -> Option<&mut ManagedTexture> {
        self.textures.get_index_mut(i).map(|(_, v)| v)
    }

    pub fn resize(&mut self, device: &Device, resolution: (u32, u32)) {
        for (name, texture) in self.textures.iter_mut() {
            texture.resize(
                resolution,
                device,
                &self.bind_group_layouts,
                &self.sampler,
                texture.resolution_scale(),
                name,
            );
        }
    }

    pub fn get_bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layouts.texture
    }

    pub fn get_compute_bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layouts.compute_texture
    }

    pub fn get_compute_mut_bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layouts.compute_mut_texture
    }

    pub fn get_bind_group_layouts(&self) -> &BindGroupLayouts {
        &self.bind_group_layouts
    }
}
