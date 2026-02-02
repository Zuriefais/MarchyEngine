pub mod jfa_compute;
pub mod jfa_compute_pass_one_shot;
pub mod jfa_compute_star;
pub mod jfa_pass;

use egui_probe::EguiProbe;
use wgpu::{CommandEncoder, Device};

use crate::{
    render_passes::{
        jfa_passes::{
            jfa_compute::JfaComputePass, jfa_compute_pass_one_shot::JfaComputeOneShotPass,
            jfa_compute_star::JfaComputeStarPass, jfa_pass::JfaRenderPass,
        },
        quad_vertex::QuadVertexRenderPass,
        seed_pass::SeedRenderPass,
    },
    texture_manager::{TextureManager, textures::TextureType},
};

#[derive(Debug, Clone, Copy, EguiProbe)]
pub enum JFAMode {
    Compute,
    ComputeStar,
    ComputeOneShot,
    Fragment,
}

#[derive(Debug, Clone, Copy, EguiProbe)]
pub struct JfaRenderOptions {
    passes: u32,
    mode: JFAMode,
}

impl Default for JfaRenderOptions {
    fn default() -> Self {
        Self {
            passes: 9,
            mode: JFAMode::Compute,
        }
    }
}

pub struct JfaPassesManager {
    compute: JfaComputePass,
    compute_star: JfaComputeStarPass,
    compute_one_shot: JfaComputeOneShotPass,
    fragment: JfaRenderPass,
    seed_pass: SeedRenderPass,
}

impl JfaPassesManager {
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
        let (texture1, texture2) = (
            texture_manager.create_texture(
                "JfaTexture",
                (width, height),
                device,
                TextureType::Standard,
                1.0,
            ),
            texture_manager.create_texture(
                "JfaTexture1",
                (width, height),
                device,
                TextureType::Standard,
                1.0,
            ),
        );
        Self {
            compute: JfaComputePass::new(device, texture_manager),
            compute_star: JfaComputeStarPass::new(device, texture_manager, width, height),
            compute_one_shot: JfaComputeOneShotPass::new(device, texture_manager),
            fragment: JfaRenderPass::new(
                device,
                quad_render_pass,
                texture_manager,
                texture1,
                texture2,
            ),
            seed_pass: SeedRenderPass::new(device, texture_manager, quad_render_pass),
        }
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        self.compute_star.resize(device, width, height);
    }

    pub fn render(
        &mut self,
        options: &JfaRenderOptions,
        encoder: &mut CommandEncoder,
        texture_manager: &mut TextureManager,
        quad_render_pass: &QuadVertexRenderPass,
        width: u32,
        height: u32,
    ) {
        match options.mode {
            JFAMode::Compute => {
                self.seed_pass
                    .render(encoder, texture_manager, quad_render_pass);
                self.compute
                    .render(encoder, texture_manager, options.passes, width, height);
            }
            JFAMode::ComputeStar => {
                self.seed_pass
                    .render(encoder, texture_manager, quad_render_pass);
                self.compute_star
                    .render(encoder, texture_manager, options.passes, width, height);
            }
            JFAMode::ComputeOneShot => {
                self.compute_one_shot.render(
                    encoder,
                    texture_manager,
                    options.passes,
                    width,
                    height,
                );
            }
            JFAMode::Fragment => {
                self.seed_pass
                    .render(encoder, texture_manager, quad_render_pass);
                self.fragment.multi_render(
                    encoder,
                    quad_render_pass,
                    texture_manager,
                    options.passes,
                    width,
                    height,
                );
            }
        }
    }
}
