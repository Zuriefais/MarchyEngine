use std::f32::consts::PI;

use egui::{Response, Ui};
use egui_probe::{EguiProbe, Style};
use glam::Vec3;
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
    #[egui_probe(with probe_fov)]
    FOV: f32,
    #[egui_probe(with probe_rotation)]
    rotation: f32,
    ray_origin: Vec3,
}

fn probe_rotation(value: &mut f32, ui: &mut Ui, _style: &Style) -> Response {
    ui.horizontal(|ui| {
        ui.add(
            egui::Slider::new(value, 0.0..=(2.0 * PI))
                .step_by(0.001)
                .fixed_decimals(9)
                .trailing_fill(true),
        );
    })
    .response
}

fn probe_fov(value: &mut f32, ui: &mut Ui, _style: &Style) -> Response {
    ui.horizontal(|ui| {
        ui.add(
            egui::Slider::new(value, 1.0..=120.0)
                .step_by(0.1)
                .fixed_decimals(1)
                .trailing_fill(true),
        );
    })
    .response
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            show: "Raymarching".into(), // Show scene texture directly
            rotation: 0.0,
            FOV: 1.0,
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
        self.raymarching_pass.render(
            encoder,
            &self.texture_manager,
            self.width,
            self.height,
            self.render_options.FOV,
            self.render_options.rotation,
        );
        if let Some(texture) = self.texture_manager.get_texture(&self.render_options.show) {
            self.show_pass
                .render(encoder, texture.bind_group(), view, &self.quad_render_pass);
        }
    }

    pub fn get_options(&mut self) -> &mut RenderOptions {
        &mut self.render_options
    }
}
