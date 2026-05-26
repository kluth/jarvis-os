//! ============================================================================
//! JARVIS OS GPU MMIO — Safe Memory-Mapped I/O Register Access
//! ============================================================================
//! Provides volatile read/write primitives for GPU MMIO registers.
//! All operations are volatile to prevent compiler reordering.
//! ============================================================================

use core::ptr::{read_volatile, write_volatile};

/// Safe wrapper for MMIO register access at a given physical address
#[derive(Debug, Clone, Copy)]
pub struct MmioReg {
    base: u64,
}

impl MmioReg {
    /// Create a new MMIO register block at the given physical address
    pub const fn new(base: u64) -> Self {
        Self { base }
    }

    /// Read a 32-bit register at offset (u16 version)
    #[inline]
    pub unsafe fn read32(&self, offset: u16) -> u32 {
        let addr = (self.base + offset as u64) as *const u32;
        read_volatile(addr)
    }

    /// Read a 32-bit register at offset (u32 version — for devices with large MMIO regions like Intel)
    #[inline]
    pub unsafe fn read32_u(&self, offset: u32) -> u32 {
        let addr = (self.base + offset as u64) as *const u32;
        read_volatile(addr)
    }

    /// Write a 32-bit register at offset (u16 version)
    #[inline]
    pub unsafe fn write32(&self, offset: u16, value: u32) {
        let addr = (self.base + offset as u64) as *mut u32;
        write_volatile(addr, value);
    }

    /// Write a 32-bit register at offset (u32 version — for devices with large MMIO regions like Intel)
    #[inline]
    pub unsafe fn write32_u(&self, offset: u32, value: u32) {
        let addr = (self.base + offset as u64) as *mut u32;
        write_volatile(addr, value);
    }

    /// Read a 16-bit register at offset
    #[inline]
    pub unsafe fn read16(&self, offset: u16) -> u16 {
        let addr = (self.base + offset as u64) as *const u16;
        read_volatile(addr)
    }

    /// Write a 16-bit register at offset
    #[inline]
    pub unsafe fn write16(&self, offset: u16, value: u16) {
        let addr = (self.base + offset as u64) as *mut u16;
        write_volatile(addr, value);
    }

    /// Read an 8-bit register at offset
    #[inline]
    pub unsafe fn read8(&self, offset: u16) -> u8 {
        let addr = (self.base + offset as u64) as *const u8;
        read_volatile(addr)
    }

    /// Write an 8-bit register at offset
    #[inline]
    pub unsafe fn write8(&self, offset: u16, value: u8) {
        let addr = (self.base + offset as u64) as *mut u8;
        write_volatile(addr, value);
    }

    /// Set a bitmask at offset (read-modify-write)
    #[inline]
    pub unsafe fn set_bits32(&self, offset: u16, bits: u32) {
        let val = self.read32(offset);
        self.write32(offset, val | bits);
    }

    /// Clear a bitmask at offset (read-modify-write)
    #[inline]
    pub unsafe fn clear_bits32(&self, offset: u16, bits: u32) {
        let val = self.read32(offset);
        self.write32(offset, val & !bits);
    }

    /// Wait for a bitmask to be set (with timeout)
    #[inline]
    pub unsafe fn poll_set32(&self, offset: u16, mask: u32, timeout: u32) -> bool {
        for _ in 0..timeout {
            if self.read32(offset) & mask == mask {
                return true;
            }
            core::hint::spin_loop();
        }
        false
    }

    /// Wait for a bitmask to be cleared (with timeout)
    #[inline]
    pub unsafe fn poll_clear32(&self, offset: u16, mask: u32, timeout: u32) -> bool {
        for _ in 0..timeout {
            if self.read32(offset) & mask == 0 {
                return true;
            }
            core::hint::spin_loop();
        }
        false
    }

    /// Copy from system memory to MMIO (for framebuffer uploads)
    #[inline]
    pub unsafe fn memcpy_to_mmio(&self, dst_offset: u16, src: *const u8, len: usize) {
        let dst = (self.base + dst_offset as u64) as *mut u8;
        for i in 0..len {
            write_volatile(dst.add(i), read_volatile(src.add(i)));
        }
    }
}
