// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 crateria

//! Shared memory layout and control protocol for out-of-process screensaver execution.

pub mod protocol;
pub mod shm;

pub use protocol::{IpcCommand, IpcResponse};
pub use shm::{FfiTerminalCell, SHM_MAGIC, SharedMemory, SharedMemoryHeader, compute_shm_size};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_commands() {
        let cmds = vec![
            IpcCommand::Init {
                cols: 120,
                rows: 40,
            },
            IpcCommand::TickAndDraw { dt_micros: 16666 },
            IpcCommand::SetSimulationRate { hz: 60.0 },
            IpcCommand::Stop,
        ];

        for cmd in cmds {
            let mut buf = Vec::new();
            cmd.write_to(&mut buf).unwrap();
            let decoded = IpcCommand::read_from(&buf[..]).unwrap();
            assert_eq!(cmd, decoded);
        }
    }

    #[test]
    fn test_ipc_responses() {
        let resps = vec![
            IpcResponse::Ready,
            IpcResponse::FrameReady { scanlines: true },
            IpcResponse::FrameReady { scanlines: false },
            IpcResponse::Ack,
        ];

        for resp in resps {
            let mut buf = Vec::new();
            resp.write_to(&mut buf).unwrap();
            let decoded = IpcResponse::read_from(&buf[..]).unwrap();
            assert_eq!(resp, decoded);
        }
    }

    #[test]
    fn test_shm_size() {
        let size = compute_shm_size(80, 24);
        let header_sz = std::mem::size_of::<SharedMemoryHeader>();
        let cell_sz = std::mem::size_of::<FfiTerminalCell>();
        assert_eq!(size, header_sz + 80 * 24 * cell_sz);
    }
}
