use egui_probe::EguiProbe;
use wgpu::{CommandEncoder, Device};

use crate::{
    render_passes::{
        quad_vertex::QuadVertexRenderPass,
        radiance_cascades_passes::{
            radiance_render::RadianceRenderPass,
            radiance_render_compute::RadianceRenderComputePass,
            radiance_render_old_pass::RadianceRenderOLDPass,
        },
    },
    texture_manager::{TextureManager, textures::TextureType},
};

pub mod radiance_render;
pub mod radiance_render_compute;
pub mod radiance_render_old_pass;

#[derive(Debug, Clone, Copy, EguiProbe)]
pub enum RadianceMode {
    Fragment(radiance_render::RadiansOptions),
    Compute(radiance_render_compute::RadiansOptions),
    FragmentOLD(radiance_render_old_pass::RadiansOptionsOLD),
}

#[derive(Debug, Clone, Copy, EguiProbe)]
pub struct RadianceCascadesRenderOptions {
    radiance_mode: RadianceMode,
}

impl Default for RadianceCascadesRenderOptions {
    fn default() -> Self {
        Self {
            radiance_mode: RadianceMode::Compute(Default::default()),
        }
    }
}

pub struct RadianceCascadesPassesManager {
    old_pass: RadianceRenderOLDPass,
    pass: RadianceRenderPass,
    compute: RadianceRenderComputePass,
    width: u32,
    height: u32,
}

impl RadianceCascadesPassesManager {
    pub fn new(
        device: &Device,
        width: u32,
        height: u32,
        quad_render_pass: &QuadVertexRenderPass,
        texture_manager: &mut TextureManager,
    ) -> Self {
        let _ = texture_manager.create_texture(
            "RadianceCascades",
            (width, height),
            device,
            TextureType::Standard,
            1.0,
        );

        let old_pass = RadianceRenderOLDPass::new(device, quad_render_pass, texture_manager);
        let pass = RadianceRenderPass::new(device, quad_render_pass, texture_manager);
        let compute = RadianceRenderComputePass::new(device, texture_manager);
        Self {
            old_pass,
            pass,
            compute,
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height
    }

    pub fn render(
        &mut self,
        render_options: &RadianceCascadesRenderOptions,
        encoder: &mut CommandEncoder,
        texture_manager: &mut TextureManager,
        quad_render_pass: &QuadVertexRenderPass,
    ) {
        match render_options.radiance_mode {
            RadianceMode::Fragment(options) => {
                self.pass
                    .render(encoder, options, texture_manager, quad_render_pass);
            }
            RadianceMode::Compute(options) => {
                self.compute
                    .render(encoder, texture_manager, options, self.width, self.height);
            }
            RadianceMode::FragmentOLD(options) => {
                self.old_pass
                    .render(encoder, options, texture_manager, quad_render_pass);
            }
        }
    }
}
