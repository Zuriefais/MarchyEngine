use egui_probe::EguiProbe;
use wgpu::{CommandEncoder, Device, Queue, TextureView};

use crate::render_passes::quad_vertex::QuadVertexRenderPass;
use crate::render_passes::raymarching_passes::raymarching_pass_compute::RaymarchingRenderComputePass;
use crate::render_passes::show_pass::ShowRenderPass;
use crate::texture_manager::{
    TextureManager,
    textures::{EngineTexture, TextureType},
};

#[derive(Debug, Clone, EguiProbe)]
pub struct RenderOptions {
    show: String,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            show: "Raymarching".into(), // Show scene texture directly
        }
    }
}

pub struct RenderPassManager {
    raymarching_pass: RaymarchingRenderComputePass,
    show_pass: ShowRenderPass,
    quad_render_pass: QuadVertexRenderPass,
    render_options: RenderOptions,
    texture_manager: TextureManager,
    width: u32,
    height: u32,
}

impl RenderPassManager {
    pub fn new(
        device: &Device,
        config: &wgpu::SurfaceConfiguration,
        width: u32,
        height: u32,
    ) -> RenderPassManager {
        let mut texture_manager = TextureManager::new(device);
        texture_manager.create_texture(
            "SceneTexture",
            (width, height),
            device,
            TextureType::SceneTexture,
            1.0,
        );
        texture_manager.create_texture(
            "Raymarching",
            (width, height),
            device,
            TextureType::Standard,
            1.0,
        );
        let quad_render_pass = QuadVertexRenderPass::new(device);

        let show_pass = ShowRenderPass::new(device, config, &quad_render_pass);
        let raymarching_pass = RaymarchingRenderComputePass::new(device, &mut texture_manager);
        Self {
            quad_render_pass,

            render_options: Default::default(),
            show_pass,

            texture_manager,

            width,
            height,
            raymarching_pass,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, device: &Device, _queue: &Queue) {
        puffin::profile_function!();

        // Skip invalid sizes
        if width == 0 || height == 0 {
            return;
        }

        // Skip if size hasn't changed
        if self.width == width && self.height == height {
            return;
        }

        self.texture_manager.resize(device, (width, height));
        self.width = width;
        self.height = height;
    }

    pub fn render(&mut self, view: &TextureView, encoder: &mut CommandEncoder, _device: &Device) {
        puffin::profile_function!();
        self.raymarching_pass
            .render(encoder, &self.texture_manager, self.width, self.height);
        if let Some(texture) = self.texture_manager.get_texture(&self.render_options.show) {
            self.show_pass
                .render(encoder, texture.bind_group(), view, &self.quad_render_pass);
        }
    }

    pub fn get_options(&mut self) -> &mut RenderOptions {
        &mut self.render_options
    }
}
