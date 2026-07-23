// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 crateria

use trance_api::TerminalCell;

/// FFI-safe representation of `TerminalCell` for shared memory communication.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FfiTerminalCell {
    pub ch: u32,
    pub fg_r: u8,
    pub fg_g: u8,
    pub fg_b: u8,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
    pub bold: u8,
}

impl From<TerminalCell> for FfiTerminalCell {
    fn from(cell: TerminalCell) -> Self {
        Self {
            ch: cell.ch as u32,
            fg_r: cell.fg.0,
            fg_g: cell.fg.1,
            fg_b: cell.fg.2,
            bg_r: cell.bg.0,
            bg_g: cell.bg.1,
            bg_b: cell.bg.2,
            bold: if cell.bold { 1 } else { 0 },
        }
    }
}

impl From<FfiTerminalCell> for TerminalCell {
    fn from(ffi: FfiTerminalCell) -> Self {
        Self {
            ch: std::char::from_u32(ffi.ch).unwrap_or(' '),
            fg: (ffi.fg_r, ffi.fg_g, ffi.fg_b),
            bg: (ffi.bg_r, ffi.bg_g, ffi.bg_b),
            bold: ffi.bold != 0,
        }
    }
}

#[repr(C)]
pub struct SharedMemoryHeader {
    pub magic: u32,
    pub cols: u32,
    pub rows: u32,
    pub frame_counter: u64,
}

pub const SHM_MAGIC: u32 = 0x54524e43;

pub fn compute_shm_size(cols: usize, rows: usize) -> usize {
    std::mem::size_of::<SharedMemoryHeader>() + cols * rows * std::mem::size_of::<FfiTerminalCell>()
}
