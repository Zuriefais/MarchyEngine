use glam::Vec2;
use log::info;
use wgpu::{
    BindGroup, Device, Queue, Sampler, TextureView,
};

use crate::texture_manager::{
    BindGroupLayouts,
    textures::{EngineTexture, standard::StandardTexture},
};

pub struct SceneTexture {
    texture: StandardTexture,
    texture_data: Vec<f32>,
}

impl EngineTexture for SceneTexture {
    fn view(&self) -> &TextureView {
        self.texture.view()
    }

    fn bind_group(&self) -> &BindGroup {
        self.texture.bind_group()
    }

    fn compute_bind_group(&self) -> &BindGroup {
        self.texture.compute_bind_group()
    }

    fn compute_mut_group_f32(&self) -> Option<&BindGroup> {
        self.texture.compute_mut_group_f32()
    }

    fn compute_mut_group_f16(&self) -> Option<&BindGroup> {
        self.texture.compute_mut_group_f16()
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
        self.texture.resolution_scale()
    }
}

impl SceneTexture {
    pub fn new(
        name: &str,
        resolution: (u32, u32),
        device: &Device,
        bind_group_layouts: &BindGroupLayouts,
        sampler: &Sampler,
        resolution_scale: f32,
    ) -> Self {
        let texture = StandardTexture::new(
            name,
            resolution,
            device,
            bind_group_layouts,
            sampler,
            resolution_scale,
        );
        let pixel_count = (resolution.0 * resolution.1) as usize;
        let flat_rgba: Vec<f32> = vec![[0f32, 0f32, 0f32, 0f32]; pixel_count]
            .into_iter()
            .flatten()
            .collect();

        // compute padded bytes per row & build a padded buffer
        let bytes_per_pixel = std::mem::size_of::<[f32; 4]>() as u32; // = 16
        let unpadded_bytes_per_row = resolution.0 * bytes_per_pixel;

        let bytes_per_row = padded_bytes_per_row(unpadded_bytes_per_row) as usize;
        let mut padded: Vec<f32> = vec![0f32; bytes_per_row * resolution.1 as usize];

        // copy each source row into the padded row
        for row in 0..resolution.1 as usize {
            let src_start = row * (resolution.0 as usize) * 4;
            let src_end = src_start + (resolution.0 as usize) * 4;
            let dst_start = row * bytes_per_row;
            padded[dst_start..dst_start + (resolution.0 as usize) * 4]
                .copy_from_slice(&flat_rgba[src_start..src_end]);
        }

        Self {
            texture,
            texture_data: flat_rgba,
        }
    }

    pub fn paint(
        &mut self,
        pos: Vec2,
        color: [f32; 4],
        brush_radius: u32,
        width: u32,
        height: u32,
        queue: &Queue,
    ) {
        let center_x = pos.x as i32;
        let center_y = pos.y as i32;
        let radius_sq = (brush_radius as i32).pow(2);

        info!("Painting at {}, {}, color: {:?}", center_x, center_y, color);

        let min_x = (center_x - brush_radius as i32).max(0) as u32;
        let max_x = (center_x + brush_radius as i32).min((width - 1) as i32) as u32;
        let min_y = (center_y - brush_radius as i32).max(0) as u32;
        let max_y = (center_y + brush_radius as i32).min((height - 1) as i32) as u32;

        if min_x > max_x || min_y > max_y {
            return;
        }

        let rect_width = max_x - min_x + 1;
        let rect_height = max_y - min_y + 1;

        let bytes_per_pixel = std::mem::size_of::<[f32; 4]>() as u32; // = 16
        let unpadded_bytes_per_row = rect_width * bytes_per_pixel;
        let bytes_per_row = padded_bytes_per_row(unpadded_bytes_per_row);

        let f32s_per_row = (bytes_per_row / std::mem::size_of::<f32>() as u32) as usize;
        let mut patch_data = vec![0f32; f32s_per_row * rect_height as usize];

        for y_in_rect in 0..rect_height {
            for x_in_rect in 0..rect_width {
                let tex_x = min_x + x_in_rect;
                let tex_y = min_y + y_in_rect;

                let dx = tex_x as i32 - center_x;
                let dy = tex_y as i32 - center_y;

                let patch_idx_start = y_in_rect as usize * f32s_per_row + x_in_rect as usize * 4;

                if dx * dx + dy * dy <= radius_sq {
                    patch_data[patch_idx_start..patch_idx_start + 4].copy_from_slice(&color);
                    let cpu_idx = ((tex_y * width) + tex_x) as usize * 4;
                    self.texture_data[cpu_idx..cpu_idx + 4].copy_from_slice(&color);
                } else {
                    // If outside, use the existing color from our CPU-side copy
                    let cpu_idx = ((tex_y * width) + tex_x) as usize * 4;
                    let old_color = &self.texture_data[cpu_idx..cpu_idx + 4];
                    patch_data[patch_idx_start..patch_idx_start + 4].copy_from_slice(old_color);
                }
            }
        }

        // 4. Write the entire patch to the GPU texture in a single call
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: min_x,
                    y: min_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&patch_data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(rect_height),
            },
            wgpu::Extent3d {
                width: rect_width,
                height: rect_height,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn padded_bytes_per_row(unpadded_row_bytes: u32) -> u32 {
    // WebGPU requires bytes_per_row be a multiple of 256
    const ALIGN: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    unpadded_row_bytes.div_ceil(ALIGN) * ALIGN
}
