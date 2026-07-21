// SPDX-License-Identifier: MIT

use super::gpu_init::GpuCell;
use trance_api::TerminalCell;

pub fn build_gpu_cells(
    grid: &[TerminalCell],
    grid_cols: usize,
    col_start: usize,
    row_start: usize,
    cols: usize,
    rows: usize,
    atlas_chars: &[char],
) -> Vec<GpuCell> {
    let mut gpu_cells = Vec::with_capacity(cols * rows);
    for row in 0..rows {
        for col in 0..cols {
            let index = (row_start + row) * grid_cols + (col_start + col);
            if let Some(cell) = grid.get(index) {
                let bg_color =
                    ((cell.bg.0 as u32) << 16) | ((cell.bg.1 as u32) << 8) | (cell.bg.2 as u32);
                let fg_color =
                    ((cell.fg.0 as u32) << 16) | ((cell.fg.1 as u32) << 8) | (cell.fg.2 as u32);
                let char_idx = if cell.ch == ' ' {
                    0xFFFFFFFF
                } else {
                    atlas_chars
                        .iter()
                        .position(|&c| c == cell.ch)
                        .map(|idx| idx as u32)
                        .unwrap_or(0xFFFFFFFF)
                };
                gpu_cells.push(GpuCell {
                    bg_color,
                    fg_color,
                    char_idx,
                    bold: u32::from(cell.bold),
                });
            } else {
                gpu_cells.push(GpuCell {
                    bg_color: 0,
                    fg_color: 0xFFFFFF,
                    char_idx: 0xFFFFFFFF,
                    bold: 0,
                });
            }
        }
    }
    gpu_cells
}

pub fn copy_staging_to_out(
    staging_buffer: &wgpu::Buffer,
    device: &wgpu::Device,
    content_w: u32,
    content_h: u32,
    unpadded: u32,
    padded: u32,
    out: &mut Vec<u8>,
) {
    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
        let _ = sender.send(v);
    });
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });

    if let Ok(Ok(())) = receiver.recv() {
        let data = buffer_slice.get_mapped_range();
        let byte_len = (content_w * content_h * 4) as usize;
        out.resize(byte_len, 0);
        for row in 0..content_h {
            let src_start = (row * padded) as usize;
            let src_end = src_start + unpadded as usize;
            let dst_start = (row * unpadded) as usize;
            let dst_end = dst_start + unpadded as usize;
            if src_end <= data.len() && dst_end <= out.len() {
                out[dst_start..dst_end].copy_from_slice(&data[src_start..src_end]);
            }
        }
        drop(data);
        staging_buffer.unmap();
    } else {
        tracing::error!("Failed to map staging buffer for wgpu cell renderer");
    }
}
