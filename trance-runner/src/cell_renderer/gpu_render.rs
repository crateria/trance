// SPDX-License-Identifier: MIT

use super::gpu_init::{GpuCell, GpuCellRenderer, Uniforms};
use trance_api::TerminalCell;

impl GpuCellRenderer {
    pub fn render(
        &mut self,
        grid: &[TerminalCell],
        grid_cols: usize,
        col_start: usize,
        row_start: usize,
        cols: usize,
        rows: usize,
        scanlines: bool,
        cell_width: usize,
        cell_height: usize,
        atlas_cols: usize,
        atlas_rows: usize,
        atlas_image: &[u8],
        atlas_dirty: bool,
        atlas_chars: &[char],
        out: &mut Vec<u8>,
    ) {
        let (content_w, content_h) = ((cols * cell_width) as u32, (rows * cell_height) as u32);
        if content_w == 0 || content_h == 0 {
            return;
        }

        let unpadded = content_w * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded + (align - unpadded % align) % align;

        let mut recreate_bg = false;
        if self.target_width != content_w
            || self.target_height != content_h
            || self.texture.is_none()
        {
            self.target_width = content_w;
            self.target_height = content_h;
            Self::ensure_texture(
                &self.device,
                &mut self.texture,
                "cell render target",
                content_w,
                content_h,
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            );
            self.staging_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("staging"),
                size: (padded * content_h) as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }));
            recreate_bg = true;
        }

        let cells_size = (cols * rows * std::mem::size_of::<GpuCell>()) as u64;
        let (cells_buf, c_re) = Self::ensure_buffer(
            &self.device,
            &mut self.cells_buffer,
            "cells",
            cells_size,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );

        let (uni_buf, u_re) = Self::ensure_buffer(
            &self.device,
            &mut self.uniform_buffer,
            "uniforms",
            std::mem::size_of::<Uniforms>() as u64,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        let (atlas_w, atlas_h) = (atlas_cols * cell_width, atlas_rows * cell_height);
        let mut a_re = false;
        if atlas_dirty
            || self.atlas_texture.is_none()
            || self.atlas_width != atlas_w
            || self.atlas_height != atlas_h
        {
            self.atlas_width = atlas_w;
            self.atlas_height = atlas_h;
            Self::ensure_texture(
                &self.device,
                &mut self.atlas_texture,
                "atlas",
                atlas_w as u32,
                atlas_h as u32,
                wgpu::TextureFormat::R8Unorm,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            );
            a_re = true;
        }

        let recreate_bind = recreate_bg || c_re || u_re || a_re;

        if (atlas_dirty || recreate_bind)
            && let Some(ref atlas_tex) = self.atlas_texture
        {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: atlas_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                atlas_image,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(atlas_w as u32),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: atlas_w as u32,
                    height: atlas_h as u32,
                    depth_or_array_layers: 1,
                },
            );
        }

        if recreate_bind || self.bind_group.is_none() {
            let Some(atlas) = self.atlas_texture.as_ref() else {
                return;
            };
            let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor::default());
            self.bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bind group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uni_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: cells_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&atlas_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                    },
                ],
            }));
        }

        let uniforms = Uniforms {
            cols: cols as u32,
            rows: rows as u32,
            cell_width: cell_width as u32,
            cell_height: cell_height as u32,
            atlas_cols: atlas_cols as u32,
            atlas_rows: atlas_rows as u32,
            scanlines: u32::from(scanlines),
            padding: 0,
        };
        self.queue
            .write_buffer(&uni_buf, 0, bytemuck::bytes_of(&uniforms));

        let gpu_cells = super::gpu_cells::build_gpu_cells(
            grid,
            grid_cols,
            col_start,
            row_start,
            cols,
            rows,
            atlas_chars,
        );
        self.queue
            .write_buffer(&cells_buf, 0, bytemuck::cast_slice(&gpu_cells));

        let Some(target_tex) = self.texture.as_ref() else {
            return;
        };
        let Some(bind_gp) = self.bind_group.as_ref() else {
            return;
        };
        let Some(staging_buf) = self.staging_buffer.as_ref() else {
            return;
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render"),
            });
        {
            let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, bind_gp, &[]);
            render_pass.draw(0..6, 0..(cols * rows) as u32);
        }

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: target_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: staging_buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: content_w,
                height: content_h,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        if let Some(ref buf) = self.staging_buffer {
            super::gpu_cells::copy_staging_to_out(
                buf,
                &self.device,
                content_w,
                content_h,
                unpadded,
                padded,
                out,
            );
        }
    }
}
