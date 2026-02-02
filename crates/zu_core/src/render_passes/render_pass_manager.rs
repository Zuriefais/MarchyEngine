use egui_probe::EguiProbe;
use wgpu::{CommandEncoder, Device, Queue, TextureView};

use crate::{
    render_passes::{
        jfa_passes::JfaRenderOptions,
        quad_vertex::QuadVertexRenderPass,
        radiance_cascades_passes::RadianceCascadesRenderOptions,
        show_pass::ShowRenderPass,
    },
    texture_manager::{
        TextureManager,
        textures::{EngineTexture, TextureType},
    },
};

#[derive(Debug, Clone, EguiProbe)]
pub struct RenderOptions {
    pub enable_lighting: bool,
    radiance_options: RadianceCascadesRenderOptions,
    jfa_options: JfaRenderOptions,
    show: String,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            enable_lighting: false, // Disable lighting to see objects directly
            radiance_options: Default::default(),
            jfa_options: Default::default(),
            show: "SceneTexture".into(), // Show scene texture directly
        }
    }
}

pub struct RenderPassManager {
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
        let quad_render_pass = QuadVertexRenderPass::new(device);

        let show_pass = ShowRenderPass::new(device, config, &quad_render_pass);

        Self {
            quad_render_pass,

            render_options: Default::default(),
            show_pass,

            texture_manager,

            width,
            height,
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

        if let Some(texture) = self.texture_manager.get_texture(&self.render_options.show) {
            self.show_pass
                .render(encoder, texture.bind_group(), view, &self.quad_render_pass);
        }
    }

    pub fn get_options(&mut self) -> &mut RenderOptions {
        &mut self.render_options
    }
}
