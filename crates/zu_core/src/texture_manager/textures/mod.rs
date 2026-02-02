use crate::texture_manager::{
    BindGroupLayouts,
    textures::{
        scene_texture::SceneTexture, standard::StandardTexture, standard_f16::StandardTextureF16,
    },
};

use wgpu::{
    BindGroup, Device, Sampler, TextureView,
};

pub mod scene_texture;
pub mod standard;
pub mod standard_f16;

pub enum TextureType {
    Standard,
    StandardF16,
    SceneTexture,
}

pub trait EngineTexture {
    fn view(&self) -> &TextureView;
    fn bind_group(&self) -> &BindGroup;
    fn compute_bind_group(&self) -> &BindGroup;
    fn compute_mut_group_f32(&self) -> Option<&BindGroup>;
    fn compute_mut_group_f16(&self) -> Option<&BindGroup>;
    fn resize(
        &mut self,
        resolution: (u32, u32),
        device: &Device,
        bind_group_layouts: &BindGroupLayouts,
        sampler: &Sampler,
        resolution_scale: f32,
        name: &str,
    );
    fn resolution_scale(&self) -> f32;
}

pub enum ManagedTexture {
    Standart(StandardTexture),
    StandartF16(StandardTextureF16),
    SceneTexture(SceneTexture),
}

impl EngineTexture for ManagedTexture {
    fn view(&self) -> &TextureView {
        self.as_engine_texture().view()
    }
    fn bind_group(&self) -> &BindGroup {
        self.as_engine_texture().bind_group()
    }

    fn compute_bind_group(&self) -> &BindGroup {
        self.as_engine_texture().compute_bind_group()
    }

    fn compute_mut_group_f32(&self) -> Option<&BindGroup> {
        self.as_engine_texture().compute_mut_group_f32()
    }

    fn compute_mut_group_f16(&self) -> Option<&BindGroup> {
        self.as_engine_texture().compute_mut_group_f16()
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
        self.as_engine_texture_mut().resize(
            resolution,
            device,
            bind_group_layouts,
            sampler,
            resolution_scale,
            name,
        );
    }

    fn resolution_scale(&self) -> f32 {
        self.as_engine_texture().resolution_scale()
    }
}

impl ManagedTexture {
    pub fn new(
        name: &str,
        resolution: (u32, u32),
        device: &Device,
        bind_group_layouts: &BindGroupLayouts,
        sampler: &Sampler,
        resolution_scale: f32,
        texture_type: TextureType,
    ) -> Self {
        match texture_type {
            TextureType::Standard => ManagedTexture::Standart(StandardTexture::new(
                name,
                resolution,
                device,
                bind_group_layouts,
                sampler,
                resolution_scale,
            )),
            TextureType::StandardF16 => ManagedTexture::StandartF16(StandardTextureF16::new(
                name,
                resolution,
                device,
                bind_group_layouts,
                sampler,
                resolution_scale,
            )),
            TextureType::SceneTexture => ManagedTexture::SceneTexture(SceneTexture::new(
                name,
                resolution,
                device,
                bind_group_layouts,
                sampler,
                resolution_scale,
            )),
        }
    }

    pub fn as_engine_texture(&self) -> &dyn EngineTexture {
        match self {
            ManagedTexture::Standart(standard) => standard,
            ManagedTexture::StandartF16(standard_f16) => standard_f16,
            ManagedTexture::SceneTexture(scene_texture) => scene_texture,
        }
    }

    pub fn as_engine_texture_mut(&mut self) -> &mut dyn EngineTexture {
        match self {
            ManagedTexture::Standart(standard) => standard,
            ManagedTexture::StandartF16(standard_f16) => standard_f16,
            ManagedTexture::SceneTexture(scene_texture) => scene_texture,
        }
    }

    pub fn standard(&self) -> Option<&StandardTexture> {
        if let ManagedTexture::Standart(standart) = self {
            Some(standart)
        } else {
            None
        }
    }

    pub fn standard_f16(&self) -> Option<&StandardTextureF16> {
        if let ManagedTexture::StandartF16(standart) = self {
            Some(standart)
        } else {
            None
        }
    }

    pub fn scene(&self) -> Option<&SceneTexture> {
        if let ManagedTexture::SceneTexture(scene) = self {
            Some(scene)
        } else {
            None
        }
    }
}
