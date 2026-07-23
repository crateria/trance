// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 crateria

use std::ffi::CString;
use std::ptr;

use crate::ffi_cell::{FfiTerminalCell, SharedMemoryHeader};

pub struct SharedMemory {
    name: String,
    fd: libc::c_int,
    ptr: *mut libc::c_void,
    size: usize,
    is_owner: bool,
    is_memfd: bool,
}

impl SharedMemory {
    pub fn create(name: &str, size: usize) -> Result<Self, String> {
        let c_name = CString::new(name).map_err(|e| e.to_string())?;

        // Attempt anonymous memfd_create (Linux 3.17+) to eliminate named SHM leak risk
        let mut fd = unsafe { libc::memfd_create(c_name.as_ptr(), libc::MFD_CLOEXEC) };
        let is_memfd = fd >= 0;

        if fd < 0 {
            unsafe {
                libc::shm_unlink(c_name.as_ptr());
            }

            fd = unsafe {
                libc::shm_open(
                    c_name.as_ptr(),
                    libc::O_CREAT | libc::O_RDWR | libc::O_EXCL,
                    0o600,
                )
            };
            if fd < 0 {
                return Err(format!(
                    "shm_open (create) failed: {}",
                    std::io::Error::last_os_error()
                ));
            }
        }

        if unsafe { libc::ftruncate(fd, size as libc::off_t) } < 0 {
            let err = std::io::Error::last_os_error();
            unsafe {
                libc::close(fd);
                if !is_memfd {
                    libc::shm_unlink(c_name.as_ptr());
                }
            }
            return Err(format!("ftruncate failed: {err}"));
        }

        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            let err = std::io::Error::last_os_error();
            unsafe {
                libc::close(fd);
                libc::shm_unlink(c_name.as_ptr());
            }
            return Err(format!("mmap failed: {err}"));
        }

        Ok(Self {
            name: name.to_string(),
            fd,
            ptr,
            size,
            is_owner: true,
            is_memfd,
        })
    }

    pub fn open(name: &str, size: usize) -> Result<Self, String> {
        let c_name = CString::new(name).map_err(|e| e.to_string())?;

        let fd = unsafe { libc::shm_open(c_name.as_ptr(), libc::O_RDWR, 0) };
        if fd < 0 {
            return Err(format!(
                "shm_open (open) failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            let err = std::io::Error::last_os_error();
            unsafe {
                libc::close(fd);
            }
            return Err(format!("mmap failed: {err}"));
        }

        Ok(Self {
            name: name.to_string(),
            fd,
            ptr,
            size,
            is_owner: false,
            is_memfd: false,
        })
    }

    pub fn fd(&self) -> libc::c_int {
        self.fd
    }

    pub fn ptr(&self) -> *mut libc::c_void {
        self.ptr
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// # Safety
    /// Caller must ensure shared memory region is validly mapped and non-null.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn header_mut(&self) -> &mut SharedMemoryHeader {
        unsafe { &mut *(self.ptr as *mut SharedMemoryHeader) }
    }

    /// # Safety
    /// Caller must ensure shared memory cells buffer is valid for `cols * rows` elements.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn cells_mut(&self) -> &mut [FfiTerminalCell] {
        let header = unsafe { self.header_mut() };
        let count = (header.cols * header.rows) as usize;
        let cells_ptr = unsafe {
            (self.ptr as *mut u8).add(std::mem::size_of::<SharedMemoryHeader>())
                as *mut FfiTerminalCell
        };
        unsafe { std::slice::from_raw_parts_mut(cells_ptr, count) }
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() && self.ptr != libc::MAP_FAILED {
                libc::munmap(self.ptr, self.size);
            }
            if self.fd >= 0 {
                libc::close(self.fd);
            }
            if self.is_owner
                && !self.is_memfd
                && let Ok(c_name) = CString::new(self.name.clone())
            {
                libc::shm_unlink(c_name.as_ptr());
            }
        }
    }
}
