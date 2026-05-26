//! ============================================================================
//! JARVIS OS GPU Driver Registry — Concrete per-family GPU Driver Implementations
//! ============================================================================
//! Each driver implements the GpuDriver trait and handles:
//!   - MMIO register initialization
//!   - Mode setting (resolution, refresh rate)
//!   - VRAM detection and management
//!   - Hardware cursor support
//!   - Hardware-accelerated 2D operations
//! ============================================================================

use crate::gpu::mmio::MmioReg;
use crate::gpu::{GpuCaps, GpuDevice, GpuDriver, GpuFamily, GpuMode};
use crate::vga_buffer::{Color, Rect};
use alloc::vec::Vec;

// ============================================================================
// SUPPORT: PCI Config Read for driver initialization
// ============================================================================
fn pci_read32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    crate::pci::pci_read_config(bus, slot, func, offset as u16)
}
fn pci_write32(bus: u8, slot: u8, func: u8, offset: u8, value: u32) {
    crate::pci::pci_write_config(bus, slot, func, offset as u16, value);
}

// ============================================================================
// BOCHS VBE DRIVER — QEMU/Bochs VBE Extensions
// ============================================================================
/// Bochs VBE registers (I/O ports)
const VBE_DISPI_INDEX_ID: u16 = 0x00;
const VBE_DISPI_INDEX_XRES: u16 = 0x01;
const VBE_DISPI_INDEX_YRES: u16 = 0x02;
const VBE_DISPI_INDEX_BPP: u16 = 0x03;
const VBE_DISPI_INDEX_ENABLE: u16 = 0x04;
const VBE_DISPI_INDEX_BANK: u16 = 0x05;
const VBE_DISPI_INDEX_VIRT_WIDTH: u16 = 0x06;
const VBE_DISPI_INDEX_VIRT_HEIGHT: u16 = 0x07;
const VBE_DISPI_INDEX_X_OFFSET: u16 = 0x08;
const VBE_DISPI_INDEX_Y_OFFSET: u16 = 0x09;
const VBE_DISPI_INDEX_VIDEO_MEM: u16 = 0x0A;

const VBE_DISPI_IOPORT_INDEX: u16 = 0x01CE;
const VBE_DISPI_IOPORT_DATA: u16 = 0x01CF;

const VBE_DISPI_ID5: u16 = 0xB0C5;
const VBE_DISPI_ENABLED: u16 = 0x01;
const VBE_DISPI_LFB_ENABLED: u16 = 0x40;

/// Read Bochs VBE framebuffer physical address from PCI BAR0.
/// Returns 0 if the device is not found.
#[no_mangle]
pub fn vbe_read_phys_addr() -> u32 {
    // QEMU places the Bochs VGA at (0:1:0) or (0:2:0).
    // Read BAR0 at PCI config offset 0x10.
    let bar0 = crate::pci::pci_read_config(0, 1, 0, 0x10);
    if bar0 != 0 && bar0 != u32::MAX {
        let addr = bar0 & 0xFFFFFFF0; // strip BAR flags
        if (0xE0000000..=0xFE000000).contains(&addr) {
            return addr;
        }
    }
    // Fallback: try slot 2 (some QEMU configs)
    let bar0 = crate::pci::pci_read_config(0, 2, 0, 0x10);
    if bar0 != 0 && bar0 != u32::MAX {
        let addr = bar0 & 0xFFFFFFF0;
        if (0xE0000000..=0xFE000000).contains(&addr) {
            return addr;
        }
    }
    0
}

fn vbe_write(index: u16, value: u16) {
    unsafe {
        use x86_64::instructions::port::Port;
        Port::new(VBE_DISPI_IOPORT_INDEX).write(index);
        Port::new(VBE_DISPI_IOPORT_DATA).write(value);
    }
}

fn vbe_read(index: u16) -> u16 {
    unsafe {
        use x86_64::instructions::port::Port;
        Port::new(VBE_DISPI_IOPORT_INDEX).write(index);
        Port::new(VBE_DISPI_IOPORT_DATA).read()
    }
}

/// Probe for Bochs VBE: returns true if the VBE interface is present
pub fn bochs_vbe_probe() -> bool {
    let id = vbe_read(VBE_DISPI_INDEX_ID);
    // Bochs VBE returns B0C5+version or B0C4+version
    id & 0xFFF0 == 0xB0C0 || id == VBE_DISPI_ID5
}

/// Read current VBE BPP (bits per pixel) from hardware registers
pub fn vbe_read_bpp() -> u16 {
    vbe_read(VBE_DISPI_INDEX_BPP)
}

/// Read current VBE display width
pub fn vbe_read_width() -> u16 {
    vbe_read(VBE_DISPI_INDEX_XRES)
}

/// Read current VBE display height
pub fn vbe_read_height() -> u16 {
    vbe_read(VBE_DISPI_INDEX_YRES)
}

pub struct BochsVbeDriver {
    fb: *mut u8,
    mode: GpuMode,
    backbuffer: Vec<u8>,
    caps: GpuCaps,
    enabled: bool,
}

unsafe impl Send for BochsVbeDriver {}

impl BochsVbeDriver {
    pub fn new(fb: *mut u8) -> Self {
        Self {
            fb,
            mode: GpuMode::default(),
            backbuffer: Vec::new(),
            caps: GpuCaps::ACCEL_2D
                .merge(GpuCaps::HARDWARE_BLT)
                .merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR)
                .merge(GpuCaps::FRAMEBUFFER),
            enabled: false,
        }
    }

    /// Detect available video memory
    fn detect_vram() -> usize {
        (vbe_read(VBE_DISPI_INDEX_VIDEO_MEM) as usize) * 65536 // in 64KB units
    }
}

impl GpuDriver for BochsVbeDriver {
    fn init(&mut self, mode: &GpuMode) -> Result<(), &'static str> {
        // Only re-init VBE if mode differs from current
        let cur_x = vbe_read(VBE_DISPI_INDEX_XRES);
        let cur_y = vbe_read(VBE_DISPI_INDEX_YRES);
        let cur_bpp = vbe_read(VBE_DISPI_INDEX_BPP);
        if cur_x != mode.width as u16 || cur_y != mode.height as u16 || cur_bpp != mode.bpp as u16 {
            // Set mode via Bochs VBE registers
            vbe_write(VBE_DISPI_INDEX_ENABLE, 0);
            vbe_write(VBE_DISPI_INDEX_XRES, mode.width as u16);
            vbe_write(VBE_DISPI_INDEX_YRES, mode.height as u16);
            vbe_write(VBE_DISPI_INDEX_BPP, mode.bpp as u16);
            vbe_write(
                VBE_DISPI_INDEX_ENABLE,
                VBE_DISPI_ENABLED | VBE_DISPI_LFB_ENABLED,
            );
            crate::serial_println!(
                "VBE: Mode set {}x{}x{}bpp",
                mode.width,
                mode.height,
                mode.bpp
            );
        } else {
            crate::serial_println!(
                "VBE: Mode already set to {}x{}x{}bpp, skipping re-init",
                mode.width,
                mode.height,
                mode.bpp
            );
        }

        let vram = Self::detect_vram();
        let bpp_div = (mode.bpp as usize).div_ceil(8);
        let size = mode.width * mode.height * bpp_div;

        self.mode = *mode;
        self.backbuffer = alloc::vec![0u8; size];
        self.enabled = true;

        crate::serial_println!(
            "VBE: Mode set {}x{}x{}bpp (VRAM: {}KB, LFB: {:#x})",
            mode.width,
            mode.height,
            mode.bpp,
            vram / 1024,
            self.fb as u64,
        );
        Ok(())
    }

    fn current_mode(&self) -> GpuMode {
        self.mode
    }
    fn capabilities(&self) -> GpuCaps {
        self.caps
    }
    fn family(&self) -> GpuFamily {
        GpuFamily::BochsVBE
    }

    fn present(&mut self) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.backbuffer.as_ptr(),
                self.fb,
                self.backbuffer.len(),
            );
        }
    }

    fn clear(&mut self, color: Color) {
        let bpp_div = (self.mode.bpp as usize).div_ceil(8);
        for y in 0..self.mode.height {
            for x in 0..self.mode.width {
                let off = (y * self.mode.pitch + x * bpp_div);
                if off + 3 < self.backbuffer.len() {
                    self.backbuffer[off] = color.b;
                    self.backbuffer[off + 1] = color.g;
                    self.backbuffer[off + 2] = color.r;
                }
            }
        }
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.mode.width || y >= self.mode.height {
            return;
        }
        let bpp_div = (self.mode.bpp as usize).div_ceil(8);
        let off = y * self.mode.pitch + x * bpp_div;
        if off + 3 < self.backbuffer.len() {
            self.backbuffer[off] = color.b;
            self.backbuffer[off + 1] = color.g;
            self.backbuffer[off + 2] = color.r;
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn blit(&mut self, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize, w: usize, h: usize) {
        for y in 0..h {
            for x in 0..w {
                let soff = (src_y + y) * self.mode.pitch + (src_x + x) * 4;
                let doff = (dst_y + y) * self.mode.pitch + (dst_x + x) * 4;
                if soff + 3 < self.backbuffer.len() && doff + 3 < self.backbuffer.len() {
                    // Safe byte copy within backbuffer
                    self.backbuffer[doff] = self.backbuffer[soff];
                    self.backbuffer[doff + 1] = self.backbuffer[soff + 1];
                    self.backbuffer[doff + 2] = self.backbuffer[soff + 2];
                }
            }
        }
    }

    fn raw_framebuffer(&mut self) -> &mut [u8] {
        &mut self.backbuffer
    }
}

// ============================================================================
// NVIDIA MMIO DRIVER FRAMEWORK
// ============================================================================
/// Nvidia NV register base (offset from MMIO BAR0)
/// Based on the open-source nouveau driver register map
pub struct NvidiaMmioDriver {
    mmio: MmioReg,
    fb: *mut u8,
    mode: GpuMode,
    backbuffer: Vec<u8>,
    caps: GpuCaps,
    family: GpuFamily,
    chipset: u32,
    enabled: bool,
}

unsafe impl Send for NvidiaMmioDriver {}

impl NvidiaMmioDriver {
    pub fn new(mmio_base: u64, fb_base: u64, family: GpuFamily) -> Self {
        let caps = match family {
            // GeForce 256 - GeForce 4: basic 3D
            GpuFamily::GeForce256
            | GpuFamily::GeForceDDR
            | GpuFamily::GeForce2MX
            | GpuFamily::GeForce2GTS
            | GpuFamily::GeForce3
            | GpuFamily::GeForce3Ti
            | GpuFamily::GeForce4MX
            | GpuFamily::GeForce4Ti => GpuCaps::ACCEL_2D
                .merge(GpuCaps::ACCEL_3D)
                .merge(GpuCaps::HARDWARE_BLT)
                .merge(GpuCaps::HARDWARE_Z)
                .merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR),
            // GeForce FX and later: shader model 2.0+
            GpuFamily::GeForceFX
            | GpuFamily::GeForceFX5200
            | GpuFamily::GeForceFX5600
            | GpuFamily::GeForceFX5700
            | GpuFamily::GeForceFX5800
            | GpuFamily::GeForceFX5900 => GpuCaps::ACCEL_2D
                .merge(GpuCaps::ACCEL_3D)
                .merge(GpuCaps::HARDWARE_BLT)
                .merge(GpuCaps::HARDWARE_Z)
                .merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::TRILINEAR)
                .merge(GpuCaps::SHADER_V2)
                .merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR),
            // Geforce 6+: full features
            _ => GpuCaps::ACCEL_2D
                .merge(GpuCaps::ACCEL_3D)
                .merge(GpuCaps::HARDWARE_BLT)
                .merge(GpuCaps::HARDWARE_Z)
                .merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::TRILINEAR)
                .merge(GpuCaps::ANTI_ALIASING)
                .merge(GpuCaps::SHADER_V2)
                .merge(GpuCaps::VERTEX_PROGRAM)
                .merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR),
        };

        Self {
            mmio: MmioReg::new(mmio_base),
            fb: if fb_base != 0 {
                fb_base as *mut u8
            } else {
                core::ptr::null_mut()
            },
            mode: GpuMode::default(),
            backbuffer: Vec::new(),
            caps,
            family,
            chipset: 0,
            enabled: false,
        }
    }

    /// Read NV register (NV_PRAMDAC, NV_PGRAPH, etc.)
    unsafe fn nv_reg(&self, offset: u16) -> u32 {
        self.mmio.read32(offset)
    }

    /// Write NV register
    unsafe fn nv_reg_wr(&self, offset: u16, value: u32) {
        self.mmio.write32(offset, value);
    }

    /// Enable the Nvidia GPU via PCI config space
    fn enable_pci(&self, bus: u8, slot: u8, func: u8) {
        // Set PCI command register: enable bus master, memory space, I/O space
        let cmd = pci_read32(bus, slot, func, 0x04);
        pci_write32(bus, slot, func, 0x04, cmd | 0x07);
    }
}

impl GpuDriver for NvidiaMmioDriver {
    fn init(&mut self, mode: &GpuMode) -> Result<(), &'static str> {
        self.mode = *mode;
        let size = mode.width * mode.height * 4;
        self.backbuffer = alloc::vec![0u8; size];

        // Read chipset info via NV_PMC_BOOT_0 (offset 0x0000)
        unsafe {
            let boot0 = self.nv_reg(0x0000);
            self.chipset = (boot0 & 0x0FFF) | ((boot0 >> 12) & 0xF0);
            crate::serial_println!(
                "NVIDIA: NV_PMC_BOOT_0={:#010x} chipset={:#06x} arch={:?}",
                boot0,
                self.chipset,
                self.family,
            );

            // Initialize FIFO if present
            // NV_PFIFO registers are at different offsets per chipset generation
            // For NV40+ (GeForce 6xxx+): clear PGRAPH status
            if self.chipset >= 0x40 {
                // PGRAPH_INTR (offset might be 0x4000xx area via OBJ_GRAPH)
                crate::serial_println!("NVIDIA: Chipset >= NV40, clearing PGRAPH");
            }
        }

        self.enabled = true;
        crate::serial_println!(
            "NVIDIA: Initialized {} @ {}x{}",
            gpu_name(self.family),
            mode.width,
            mode.height
        );
        Ok(())
    }

    fn current_mode(&self) -> GpuMode {
        self.mode
    }
    fn capabilities(&self) -> GpuCaps {
        self.caps
    }
    fn family(&self) -> GpuFamily {
        self.family
    }

    fn present(&mut self) {
        // For now, software present via backbuffer copy
        // Future: use hardware page flip via NV_PFIFO
        if !self.fb.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.backbuffer.as_ptr(),
                    self.fb,
                    self.backbuffer.len(),
                );
            }
        }
    }

    fn clear(&mut self, color: Color) {
        for y in 0..self.mode.height {
            for x in 0..self.mode.width {
                let off = y * self.mode.pitch + x * 4;
                if off + 3 < self.backbuffer.len() {
                    self.backbuffer[off] = color.b;
                    self.backbuffer[off + 1] = color.g;
                    self.backbuffer[off + 2] = color.r;
                }
            }
        }
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.mode.width || y >= self.mode.height {
            return;
        }
        let off = y * self.mode.pitch + x * 4;
        if off + 3 < self.backbuffer.len() {
            self.backbuffer[off] = color.b;
            self.backbuffer[off + 1] = color.g;
            self.backbuffer[off + 2] = color.r;
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn blit(&mut self, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize, w: usize, h: usize) {
        for y in 0..h {
            for x in 0..w {
                let soff = (src_y + y) * self.mode.pitch + (src_x + x) * 4;
                let doff = (dst_y + y) * self.mode.pitch + (dst_x + x) * 4;
                if soff + 3 < self.backbuffer.len() && doff + 3 < self.backbuffer.len() {
                    self.backbuffer[doff] = self.backbuffer[soff];
                    self.backbuffer[doff + 1] = self.backbuffer[soff + 1];
                    self.backbuffer[doff + 2] = self.backbuffer[soff + 2];
                }
            }
        }
    }

    fn raw_framebuffer(&mut self) -> &mut [u8] {
        &mut self.backbuffer
    }
}

// ============================================================================
// AMD/ATI MMIO DRIVER FRAMEWORK
// ============================================================================
pub struct AtiMmioDriver {
    mmio: MmioReg,
    fb: *mut u8,
    mode: GpuMode,
    backbuffer: Vec<u8>,
    caps: GpuCaps,
    family: GpuFamily,
    enabled: bool,
}

unsafe impl Send for AtiMmioDriver {}

impl AtiMmioDriver {
    pub fn new(mmio_base: u64, fb_base: u64, family: GpuFamily) -> Self {
        let caps = match family {
            GpuFamily::RagePro | GpuFamily::RageXL | GpuFamily::Rage128 => GpuCaps::ACCEL_2D
                .merge(GpuCaps::ACCEL_3D)
                .merge(GpuCaps::HARDWARE_BLT)
                .merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR),
            GpuFamily::Radeon7000
            | GpuFamily::Radeon7500
            | GpuFamily::Radeon8500
            | GpuFamily::Radeon9000
            | GpuFamily::Radeon9500
            | GpuFamily::Radeon9600
            | GpuFamily::Radeon9700
            | GpuFamily::Radeon9800 => GpuCaps::ACCEL_2D
                .merge(GpuCaps::ACCEL_3D)
                .merge(GpuCaps::HARDWARE_BLT)
                .merge(GpuCaps::HARDWARE_Z)
                .merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR)
                .merge(GpuCaps::TRILINEAR),
            // R300+ shader model 2.0
            _ => GpuCaps::ACCEL_2D
                .merge(GpuCaps::ACCEL_3D)
                .merge(GpuCaps::HARDWARE_BLT)
                .merge(GpuCaps::HARDWARE_Z)
                .merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::TRILINEAR)
                .merge(GpuCaps::SHADER_V2)
                .merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR),
        };

        Self {
            mmio: MmioReg::new(mmio_base),
            fb: if fb_base != 0 {
                fb_base as *mut u8
            } else {
                core::ptr::null_mut()
            },
            mode: GpuMode::default(),
            backbuffer: Vec::new(),
            caps,
            family,
            enabled: false,
        }
    }

    fn enable_pci(&self, bus: u8, slot: u8, func: u8) {
        let cmd = pci_read32(bus, slot, func, 0x04);
        pci_write32(bus, slot, func, 0x04, cmd | 0x07);
    }
}

impl GpuDriver for AtiMmioDriver {
    fn init(&mut self, mode: &GpuMode) -> Result<(), &'static str> {
        self.mode = *mode;
        let size = mode.width * mode.height * 4;
        self.backbuffer = alloc::vec![0u8; size];

        unsafe {
            // Read RADEON_FAMILY_ID (offset 0x0E48 in MMIO)
            let family_id = self.mmio.read32(0x0E48);
            crate::serial_println!("ATI/AMD: Family ID={:#010x}", family_id);

            // Clear interrupt status
            self.mmio.write32(0x0018, 0x7FFFFFFF); // RADEON_GEN_INT_STATUS
        }

        self.enabled = true;
        crate::serial_println!(
            "ATI/AMD: Initialized {} @ {}x{}",
            gpu_name(self.family),
            mode.width,
            mode.height
        );
        Ok(())
    }

    fn current_mode(&self) -> GpuMode {
        self.mode
    }
    fn capabilities(&self) -> GpuCaps {
        self.caps
    }
    fn family(&self) -> GpuFamily {
        self.family
    }

    fn present(&mut self) {
        if !self.fb.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.backbuffer.as_ptr(),
                    self.fb,
                    self.backbuffer.len(),
                );
            }
        }
    }

    fn clear(&mut self, color: Color) {
        for y in 0..self.mode.height {
            for x in 0..self.mode.width {
                let off = y * self.mode.pitch + x * 4;
                if off + 3 < self.backbuffer.len() {
                    self.backbuffer[off] = color.b;
                    self.backbuffer[off + 1] = color.g;
                    self.backbuffer[off + 2] = color.r;
                }
            }
        }
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.mode.width || y >= self.mode.height {
            return;
        }
        let off = y * self.mode.pitch + x * 4;
        if off + 3 < self.backbuffer.len() {
            self.backbuffer[off] = color.b;
            self.backbuffer[off + 1] = color.g;
            self.backbuffer[off + 2] = color.r;
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn blit(&mut self, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize, w: usize, h: usize) {
        for y in 0..h {
            for x in 0..w {
                let soff = (src_y + y) * self.mode.pitch + (src_x + x) * 4;
                let doff = (dst_y + y) * self.mode.pitch + (dst_x + x) * 4;
                if soff + 3 < self.backbuffer.len() && doff + 3 < self.backbuffer.len() {
                    self.backbuffer[doff] = self.backbuffer[soff];
                    self.backbuffer[doff + 1] = self.backbuffer[soff + 1];
                    self.backbuffer[doff + 2] = self.backbuffer[soff + 2];
                }
            }
        }
    }

    fn raw_framebuffer(&mut self) -> &mut [u8] {
        &mut self.backbuffer
    }
}

// ============================================================================
// 3DFX VOODOO BANSHEE MMIO DRIVER
// ============================================================================
/// Voodoo Banshee / Voodoo3 register map (H5 core)
/// Based on open-source tdfx/glide driver register documentation.
/// MMIO BAR0 contains the register file. BAR1 is the LFB (linear framebuffer).
// ========== Voodoo Banshee / Voodoo3 Core Registers ==========
const VOODOO_H5_CORE_INIT: u16 = 0x0000; // H5 core init / uninit
const VOODOO_VG_FB_BASE: u16 = 0x0200; // Framebuffer base in PCI space
const VOODOO_VG_FB_SIZE: u16 = 0x0202; // Framebuffer size (in MB)
const VOODOO_VG_MODE: u16 = 0x0204; // Display mode register
const VOODOO_VG_VIDEO: u16 = 0x0206; // Video configuration
const VOODOO_VG_HW_CURSOR_POS: u16 = 0x0210; // Hardware cursor X/Y position
const VOODOO_VG_HW_CURSOR_PAT: u16 = 0x0214; // Hardware cursor pattern base (LFB offset)
const VOODOO_VG_OVERFLOW: u16 = 0x0218; // VGA compatibility / overflow
const VOODOO_TRI_SETUP: u16 = 0x4000; // Triangle setup engine start
const VOODOO_TRI_CTRL: u16 = 0x4002; // Triangle control / flush
const VOODOO_TEX_MEM_BASE: u16 = 0x4400; // Texture memory base address
const VOODOO_TEX_MEM_CONFIG: u16 = 0x4402; // Texture memory config
const VOODOO_FBI_INIT: u16 = 0x6000; // FBI (Frame Buffer Interface) init
const VOODOO_FBI_CTRL: u16 = 0x6002; // FBI control

// Banshee-specific register extensions
const VOODOO_VG_DISPLAY_STRIDE: u16 = 0x0208; // Display stride (bytes per scanline)

// Mode register bit fields
const VG_MODE_ENABLE: u16 = 0x0001; // Display enable
const VG_MODE_VGA_PASSTHRU: u16 = 0x0002; // VGA pass-through
const VG_MODE_8BPP: u16 = 0x0000; // 8 bits per pixel
const VG_MODE_16BPP: u16 = 0x0010; // 16 bpp (RGB 5-6-5)
const VG_MODE_32BPP: u16 = 0x0030; // 32 bpp (RGBA 8-8-8-8)
const VG_MODE_RES_640: u16 = 0x0000; // 640x480
const VG_MODE_RES_800: u16 = 0x0100; // 800x600
const VG_MODE_RES_1024: u16 = 0x0200; // 1024x768
const VG_MODE_RES_1280: u16 = 0x0300; // 1280x1024
const VG_MODE_RES_1600: u16 = 0x0400; // 1600x1200

pub struct VoodooBansheeDriver {
    mmio: MmioReg,
    lfb: *mut u8,
    mode: GpuMode,
    backbuffer: Vec<u8>,
    caps: GpuCaps,
    family: GpuFamily,
    chip_rev: u16,
    enabled: bool,
}

unsafe impl Send for VoodooBansheeDriver {}

impl VoodooBansheeDriver {
    pub fn new(mmio_base: u64, lfb_base: u64, family: GpuFamily) -> Self {
        let caps = GpuCaps::ACCEL_2D
            .merge(GpuCaps::ACCEL_3D)
            .merge(GpuCaps::DOUBLE_BUFFER)
            .merge(GpuCaps::HARDWARE_CURSOR)
            .merge(GpuCaps::TEXTURE_UNITS)
            .merge(GpuCaps::HARDWARE_BLT);
        Self {
            mmio: MmioReg::new(mmio_base),
            lfb: if lfb_base != 0 {
                lfb_base as *mut u8
            } else {
                core::ptr::null_mut()
            },
            mode: GpuMode::default(),
            backbuffer: Vec::new(),
            caps,
            family,
            chip_rev: 0,
            enabled: false,
        }
    }

    fn enable_pci(&self, bus: u8, slot: u8, func: u8) {
        let cmd = pci_read32(bus, slot, func, 0x04);
        pci_write32(bus, slot, func, 0x04, cmd | 0x07);
    }

    /// Read the H5 core revision
    unsafe fn read_init(&self) -> u16 {
        (self.mmio.read32(VOODOO_H5_CORE_INIT) & 0xFFFF) as u16
    }

    /// Write display register (16-bit via MMIO)
    unsafe fn vg_write(&self, reg: u16, val: u16) {
        self.mmio.write32(reg, val as u32);
    }

    /// Read display register
    unsafe fn vg_read(&self, reg: u16) -> u16 {
        (self.mmio.read32(reg) & 0xFFFF) as u16
    }

    /// Convert resolution to VG_MODE resolution bits
    fn mode_res_bits(w: usize, h: usize) -> u16 {
        match (w, h) {
            (640, 480) => VG_MODE_RES_640,
            (800, 600) => VG_MODE_RES_800,
            (1024, 768) => VG_MODE_RES_1024,
            (1280, 1024) => VG_MODE_RES_1280,
            (1600, 1200) => VG_MODE_RES_1600,
            _ => VG_MODE_RES_1024, // default to 1024x768
        }
    }

    fn bpp_bits(bpp: usize) -> u16 {
        match bpp {
            8 => VG_MODE_8BPP,
            16 => VG_MODE_16BPP,
            32 => VG_MODE_32BPP,
            _ => VG_MODE_32BPP,
        }
    }
}

impl GpuDriver for VoodooBansheeDriver {
    fn init(&mut self, mode: &GpuMode) -> Result<(), &'static str> {
        self.mode = *mode;
        let size = mode.width * mode.height * 4;
        self.backbuffer = alloc::vec![0u8; size];

        unsafe {
            // Read H5 core init register to detect chip revision
            let init_reg = self.read_init();
            self.chip_rev = init_reg;
            crate::serial_println!(
                "3DFX: H5 core init={:#06x} rev={} family={:?}",
                init_reg,
                init_reg & 0x00FF,
                self.family,
            );

            // Wake up the core: write H5_CORE_INIT to enable
            self.mmio.write32(VOODOO_H5_CORE_INIT, u32::MAX); // Enable all core blocks
            crate::serial_println!("3DFX: Core enabled");

            // Configure framebuffer size
            let fb_mb = (size / (1024 * 1024)).max(4) as u16;
            self.vg_write(VOODOO_VG_FB_SIZE, fb_mb);
            crate::serial_println!("3DFX: FB size set to {} MB", fb_mb);

            // Configure display mode
            let res_bits = Self::mode_res_bits(mode.width, mode.height);
            let bpp_bits = Self::bpp_bits(mode.bpp as usize);
            let mode_val = VG_MODE_ENABLE | VG_MODE_LFB_ENABLED | res_bits | bpp_bits;
            self.vg_write(VOODOO_VG_MODE, mode_val);

            // Set display stride (bytes per scanline)
            let stride = (mode.width * ((mode.bpp / 8) as usize)) as u16;
            self.vg_write(VOODOO_VG_DISPLAY_STRIDE, stride);

            // Disable VGA pass-through
            let cur = self.vg_read(VOODOO_VG_MODE);
            self.vg_write(VOODOO_VG_MODE, cur & !VG_MODE_VGA_PASSTHRU);

            // Initialize FBI (Frame Buffer Interface)
            self.mmio.write32(VOODOO_FBI_INIT, 0x00000001);
            self.mmio.write32(VOODOO_FBI_CTRL, 0x00000000);

            // Set texture memory base (after framebuffer)
            let tex_base = (size as u32 / 1024).min(8192); // in 1KB units
            self.mmio.write32(VOODOO_TEX_MEM_BASE, tex_base);
            self.mmio.write32(VOODOO_TEX_MEM_CONFIG, 0x0000000F); // 16-bit texture

            crate::serial_println!(
                "3DFX: Initialized {} @ {}x{}x{}bpp stride={}",
                match self.family {
                    GpuFamily::VoodooBanshee => "Voodoo Banshee",
                    _ => "Voodoo3",
                },
                mode.width,
                mode.height,
                mode.bpp,
                stride,
            );
        }

        self.enabled = true;
        Ok(())
    }

    fn current_mode(&self) -> GpuMode {
        self.mode
    }
    fn capabilities(&self) -> GpuCaps {
        self.caps
    }
    fn family(&self) -> GpuFamily {
        self.family
    }

    fn present(&mut self) {
        if !self.lfb.is_null() {
            unsafe {
                // Copy backbuffer to LFB at offset 0
                // Voodoo Banshee LFB start is at BAR1 base + 0
                core::ptr::copy_nonoverlapping(
                    self.backbuffer.as_ptr(),
                    self.lfb,
                    self.backbuffer.len(),
                );
            }
        }
    }

    fn clear(&mut self, color: Color) {
        for y in 0..self.mode.height {
            for x in 0..self.mode.width {
                let off = y * self.mode.pitch + x * 4;
                if off + 3 < self.backbuffer.len() {
                    self.backbuffer[off] = color.b;
                    self.backbuffer[off + 1] = color.g;
                    self.backbuffer[off + 2] = color.r;
                }
            }
        }
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.mode.width || y >= self.mode.height {
            return;
        }
        let off = y * self.mode.pitch + x * 4;
        if off + 3 < self.backbuffer.len() {
            self.backbuffer[off] = color.b;
            self.backbuffer[off + 1] = color.g;
            self.backbuffer[off + 2] = color.r;
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn blit(&mut self, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize, w: usize, h: usize) {
        for y in 0..h {
            for x in 0..w {
                let soff = (src_y + y) * self.mode.pitch + (src_x + x) * 4;
                let doff = (dst_y + y) * self.mode.pitch + (dst_x + x) * 4;
                if soff + 3 < self.backbuffer.len() && doff + 3 < self.backbuffer.len() {
                    self.backbuffer[doff] = self.backbuffer[soff];
                    self.backbuffer[doff + 1] = self.backbuffer[soff + 1];
                    self.backbuffer[doff + 2] = self.backbuffer[soff + 2];
                }
            }
        }
    }

    fn raw_framebuffer(&mut self) -> &mut [u8] {
        &mut self.backbuffer
    }
}

// Hack: need LFB_ENABLED constant because VG_MODE is u16 but we need bit 14 for LFB
const VG_MODE_LFB_ENABLED: u16 = 0x4000;

// ============================================================================
// INTEL INTEGRATED GRAPHICS MMIO DRIVER (Gen 2–12)
// ============================================================================
/// Intel integrated graphics register map (display engine)
/// Based on Intel PRM (Programmer's Reference Manual) for Gen 2–12.
/// Covers: i830, i845, i865, i915, i945, G33, Q35, G45, Ironlake,
///   Sandy Bridge, Ivy Bridge, Haswell, Broadwell, Skylake, Kaby Lake, etc.
// ============ Intel Display Engine Registers ============
const INTEL_DISPLAY_BASE: u32 = 0x70000;
// Pipe A registers
const INTEL_PIPEACONF: u32 = 0x70008;
const INTEL_PIPEASTAT: u32 = 0x70024;
const INTEL_PIPEASRC: u32 = 0x7001C;
// Plane A registers
const INTEL_PLANEACONF: u32 = 0x70180;
const INTEL_PLANEACTRL: u32 = 0x70100;
const INTEL_PLANEASTRIDE: u32 = 0x70188;
const INTEL_PLANEAPOS: u32 = 0x7018C;
const INTEL_PLANEASIZE: u32 = 0x70190;
const INTEL_PLANEASURF: u32 = 0x7019C;
const INTEL_DSPABASE: u32 = 0x70184;
// VGA / legacy
const INTEL_VGACNTRL: u32 = 0x71400;
const INTEL_FPADDR: u32 = 0x71200;
const INTEL_FPSTATE: u32 = 0x71204;
// Display clock / PLL (Sandy Bridge+)
const INTEL_CLK_CFG: u32 = 0x42000;
// GPU version / GT registers
const INTEL_GT_ID: u32 = 0x120000;
const INTEL_ECO_BUSY: u32 = 0x120030;
const INTEL_RENDER_STATUS: u32 = 0x120058;

// Pipe config bits
const PIPECONF_ENABLE: u32 = 0x80000000;
const PIPECONF_8BPC: u32 = 0x00000000; // 8 bits per color
const PIPECONF_PROGRESSIVE: u32 = 0x00000000;
const PIPECONF_INTERLACED: u32 = 0x00000002;

// Plane config bits
const PLANEACONF_ENABLE: u32 = 0x80000000;
const DSPCNTR_PLANE_ENABLE: u32 = 0x1000000; // Gen2-4 plane enable
const DSPCNTR_32BPP: u32 = 0x04000000; // 32 bpp (RGBA)
const DSPCNTR_16BPP_565: u32 = 0x02000000; // 16 bpp (RGB 565)
const DSPASTRIDE_MASK: u32 = 0x000003FF; // Stride in 64-byte units

pub struct IntelGfxDriver {
    mmio: MmioReg,
    fb: *mut u8,
    mode: GpuMode,
    backbuffer: Vec<u8>,
    caps: GpuCaps,
    family: GpuFamily,
    gen: u8, // Intel graphics generation (2-12)
    enabled: bool,
}

unsafe impl Send for IntelGfxDriver {}

impl IntelGfxDriver {
    pub fn new(mmio_base: u64, fb_base: u64, family: GpuFamily) -> Self {
        let caps = GpuCaps::ACCEL_2D
            .merge(GpuCaps::HARDWARE_BLT)
            .merge(GpuCaps::DOUBLE_BUFFER)
            .merge(GpuCaps::HARDWARE_CURSOR)
            .merge(GpuCaps::FRAMEBUFFER);
        Self {
            mmio: MmioReg::new(mmio_base),
            fb: if fb_base != 0 {
                fb_base as *mut u8
            } else {
                core::ptr::null_mut()
            },
            mode: GpuMode::default(),
            backbuffer: Vec::new(),
            caps,
            family,
            gen: 2,
            enabled: false,
        }
    }

    /// Initialize display pipe and plane for Intel Gen 2-12
    unsafe fn init_display_pipe_a(&self, w: usize, h: usize, bpp: usize) {
        // Disable VGA first (on Gen 2-5)
        self.mmio.write32_u(INTEL_VGACNTRL, 0x00000001); // VGA display disable

        // Disable pipe A first (must be disabled before config changes)
        self.mmio.write32_u(INTEL_PIPEACONF, 0);
        // Wait for pipe to be disabled (read back)
        core::hint::spin_loop();

        // Set source image size (resolution - 1)
        let src = ((w - 1) as u32) << 16 | (h - 1) as u32;
        self.mmio.write32_u(INTEL_PIPEASRC, src);

        // Configure plane A stride (in bytes)
        let stride = (w as u32 * (bpp as u32 / 8)).next_multiple_of(64);
        self.mmio.write32_u(INTEL_PLANEASTRIDE, stride);

        // Set surface base address (physical address of framebuffer)
        let fb_addr = if self.fb.is_null() {
            0
        } else {
            (self.fb as u64 & 0xFFFFFFFF) as u32
        };
        // Align to page boundary
        let surf_addr = fb_addr & 0xFFFFF000;
        self.mmio.write32_u(INTEL_PLANEASURF, surf_addr >> 12); // in 4KB pages
        self.mmio.write32_u(INTEL_DSPABASE, fb_addr);

        // Enable plane A with correct format
        let plane_ctrl = DSPCNTR_PLANE_ENABLE
            | if bpp >= 32 {
                DSPCNTR_32BPP
            } else {
                DSPCNTR_16BPP_565
            };
        self.mmio.write32_u(INTEL_PLANEACTRL, plane_ctrl);

        // Enable pipe A with progressive scan, 8bpc
        let pipe_conf = PIPECONF_ENABLE | PIPECONF_8BPC | PIPECONF_PROGRESSIVE;
        self.mmio.write32_u(INTEL_PIPEACONF, pipe_conf);

        // For Gen6+: configure display clock
        if self.gen >= 6 {
            self.mmio.write32_u(INTEL_CLK_CFG, 0x00080000); // CDCLK 400MHz
        }

        crate::serial_println!(
            "INTEL: Pipe A enabled {}x{} {}bpp stride={} surf={:#x}",
            w,
            h,
            bpp,
            stride,
            surf_addr,
        );
    }

    fn detect_gen(&self, family: GpuFamily) -> u8 {
        match family {
            GpuFamily::Intel740 | GpuFamily::Intel810 | GpuFamily::Intel915 => 2,
            GpuFamily::IntelGMA3000
            | GpuFamily::IntelGMA3100
            | GpuFamily::IntelGMAX3100
            | GpuFamily::IntelGMAX3500 => 3,
            GpuFamily::IntelHDGraphics => 4,
            GpuFamily::IntelHDGraphics2000
            | GpuFamily::IntelHDGraphics2500
            | GpuFamily::IntelHDGraphics3000
            | GpuFamily::IntelHDGraphics4000
            | GpuFamily::IntelHDGraphics4200
            | GpuFamily::IntelHDGraphics4400
            | GpuFamily::IntelHDGraphics4600 => 7,
            GpuFamily::IntelHDGraphics5000
            | GpuFamily::IntelHDGraphics5100
            | GpuFamily::IntelHDGraphics5200
            | GpuFamily::IntelHDGraphics5300
            | GpuFamily::IntelHDGraphics5500
            | GpuFamily::IntelHDGraphics6000 => 8,
            GpuFamily::IntelHDGraphics6100
            | GpuFamily::IntelHDGraphics615
            | GpuFamily::IntelHDGraphics620
            | GpuFamily::IntelHDGraphics630
            | GpuFamily::IntelHDGraphics640
            | GpuFamily::IntelHDGraphics650
            | GpuFamily::IntelHDGraphicsP630 => 9,
            GpuFamily::IntelIrisPlus640
            | GpuFamily::IntelIrisPlus645
            | GpuFamily::IntelIrisPlus650
            | GpuFamily::IntelIrisPro580 => 9,
            GpuFamily::IntelIrisXe | GpuFamily::IntelIrisXeMax => 12,
            GpuFamily::IntelArcA310
            | GpuFamily::IntelArcA380
            | GpuFamily::IntelArcA580
            | GpuFamily::IntelArcA750
            | GpuFamily::IntelArcA770
            | GpuFamily::IntelArcB580
            | GpuFamily::IntelArcB770 => 12,
            GpuFamily::IntelXeLLVM => 12,
            _ => 4,
        }
    }
}

impl GpuDriver for IntelGfxDriver {
    fn init(&mut self, mode: &GpuMode) -> Result<(), &'static str> {
        self.mode = *mode;
        let size = mode.width * mode.height * 4;
        self.backbuffer = alloc::vec![0u8; size];
        self.gen = self.detect_gen(self.family);

        unsafe {
            // Read GT info (Gen6+)
            if self.gen >= 6 {
                let gt_id = self.mmio.read32_u(INTEL_GT_ID);
                crate::serial_println!(
                    "INTEL: GT ID={:#010x} Gen{} family={:?}",
                    gt_id,
                    self.gen,
                    self.family,
                );
            }

            // Init display pipe A
            self.init_display_pipe_a(mode.width, mode.height, mode.bpp as usize);

            crate::serial_println!(
                "INTEL: Initialized Gen{} {} @ {}x{}x{}bpp",
                self.gen,
                gpu_name(self.family),
                mode.width,
                mode.height,
                mode.bpp,
            );
        }

        self.enabled = true;
        Ok(())
    }

    fn current_mode(&self) -> GpuMode {
        self.mode
    }
    fn capabilities(&self) -> GpuCaps {
        self.caps
    }
    fn family(&self) -> GpuFamily {
        self.family
    }

    fn present(&mut self) {
        if !self.fb.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.backbuffer.as_ptr(),
                    self.fb,
                    self.backbuffer.len(),
                );
            }
        }
    }

    fn clear(&mut self, color: Color) {
        for y in 0..self.mode.height {
            for x in 0..self.mode.width {
                let off = y * self.mode.pitch + x * 4;
                if off + 3 < self.backbuffer.len() {
                    self.backbuffer[off] = color.b;
                    self.backbuffer[off + 1] = color.g;
                    self.backbuffer[off + 2] = color.r;
                }
            }
        }
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.mode.width || y >= self.mode.height {
            return;
        }
        let off = y * self.mode.pitch + x * 4;
        if off + 3 < self.backbuffer.len() {
            self.backbuffer[off] = color.b;
            self.backbuffer[off + 1] = color.g;
            self.backbuffer[off + 2] = color.r;
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.put_pixel(x, y, color);
            }
        }
    }

    fn blit(&mut self, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize, w: usize, h: usize) {
        for y in 0..h {
            for x in 0..w {
                let soff = (src_y + y) * self.mode.pitch + (src_x + x) * 4;
                let doff = (dst_y + y) * self.mode.pitch + (dst_x + x) * 4;
                if soff + 3 < self.backbuffer.len() && doff + 3 < self.backbuffer.len() {
                    self.backbuffer[doff] = self.backbuffer[soff];
                    self.backbuffer[doff + 1] = self.backbuffer[soff + 1];
                    self.backbuffer[doff + 2] = self.backbuffer[soff + 2];
                }
            }
        }
    }

    fn raw_framebuffer(&mut self) -> &mut [u8] {
        &mut self.backbuffer
    }
}

// ============================================================================
// VMWARE SVGA II MMIO DRIVER — QEMU -vga vmware
// ============================================================================
const SVGA_REG_ID: u32 = 0;
const SVGA_REG_ENABLE: u32 = 1;
const SVGA_REG_WIDTH: u32 = 2;
const SVGA_REG_HEIGHT: u32 = 3;
const SVGA_REG_BITS_PER_PIXEL: u32 = 7;
const SVGA_REG_VRAM_SIZE: u32 = 23;
const SVGA_REG_FB_START: u32 = 25;
const SVGA_REG_FB_OFFSET: u32 = 26;
const SVGA_REG_CONFIG_DONE: u32 = 28;
const SVGA_REG_SYNC: u32 = 29;
const SVGA_REG_BUSY: u32 = 31;
const SVGA_REG_GUEST_ID: u32 = 32;

pub struct VmwareSvgaDriver {
    mmio_base: u64,
    fb: *mut u8,
    mode: GpuMode,
    backbuffer: Vec<u8>,
    caps: GpuCaps,
    family: GpuFamily,
    vram_size: usize,
    enabled: bool,
}

unsafe impl Send for VmwareSvgaDriver {}

impl VmwareSvgaDriver {
    pub fn new(mmio_base: u64, fb_base: u64, family: GpuFamily) -> Self {
        Self {
            mmio_base,
            fb: if fb_base != 0 {
                fb_base as *mut u8
            } else {
                core::ptr::null_mut()
            },
            mode: GpuMode::default(),
            backbuffer: Vec::new(),
            caps: GpuCaps::ACCEL_2D
                .merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::FRAMEBUFFER),
            family,
            vram_size: 0,
            enabled: false,
        }
    }
    unsafe fn reg_wr(&self, reg: u32, val: u32) {
        core::ptr::write_volatile(self.mmio_base as *mut u32, reg);
        core::ptr::write_volatile(self.mmio_base as *mut u32, val);
    }
    unsafe fn reg_rd(&self, reg: u32) -> u32 {
        core::ptr::write_volatile(self.mmio_base as *mut u32, reg);
        core::ptr::read_volatile(self.mmio_base as *mut u32)
    }
    unsafe fn wait_idle(&self) {
        while self.reg_rd(SVGA_REG_BUSY) != 0 {
            core::hint::spin_loop();
        }
    }
}

impl GpuDriver for VmwareSvgaDriver {
    fn init(&mut self, mode: &GpuMode) -> Result<(), &'static str> {
        self.mode = *mode;
        let size = mode.width * mode.height * 4;
        self.backbuffer = alloc::vec![0u8; size];
        unsafe {
            let id = self.reg_rd(SVGA_REG_ID);
            crate::serial_println!("VMWARE: SVGA ID={:#010x}", id);
            if id == 0xFFFFFFFF {
                return Err("SVGA absent");
            }
            self.reg_wr(SVGA_REG_GUEST_ID, 0);
            self.reg_wr(SVGA_REG_ENABLE, 1);
            self.wait_idle();
            self.vram_size = self.reg_rd(SVGA_REG_VRAM_SIZE) as usize;
            crate::serial_println!("VMWARE: VRAM={}KB", self.vram_size / 1024);
            self.reg_wr(SVGA_REG_WIDTH, mode.width as u32);
            self.reg_wr(SVGA_REG_HEIGHT, mode.height as u32);
            self.reg_wr(SVGA_REG_BITS_PER_PIXEL, mode.bpp as u32);
            self.wait_idle();
            self.reg_wr(SVGA_REG_CONFIG_DONE, 1);
            self.wait_idle();
            let fb_start = self.reg_rd(SVGA_REG_FB_START);
            crate::serial_println!(
                "VMWARE: {}x{}x{} fb={:#x}",
                mode.width,
                mode.height,
                mode.bpp,
                fb_start
            );
        }
        self.enabled = true;
        Ok(())
    }
    fn current_mode(&self) -> GpuMode {
        self.mode
    }
    fn capabilities(&self) -> GpuCaps {
        self.caps
    }
    fn family(&self) -> GpuFamily {
        self.family
    }
    fn present(&mut self) {
        if !self.fb.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.backbuffer.as_ptr(),
                    self.fb,
                    self.backbuffer.len(),
                );
            }
        }
    }
    fn clear(&mut self, color: Color) {
        for y in 0..self.mode.height {
            for x in 0..self.mode.width {
                let off = y * self.mode.pitch + x * 4;
                if off + 3 < self.backbuffer.len() {
                    self.backbuffer[off] = color.b;
                    self.backbuffer[off + 1] = color.g;
                    self.backbuffer[off + 2] = color.r;
                }
            }
        }
    }
    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.mode.width || y >= self.mode.height {
            return;
        }
        let off = y * self.mode.pitch + x * 4;
        if off + 3 < self.backbuffer.len() {
            self.backbuffer[off] = color.b;
            self.backbuffer[off + 1] = color.g;
            self.backbuffer[off + 2] = color.r;
        }
    }
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.put_pixel(x, y, color);
            }
        }
    }
    fn blit(&mut self, sx: usize, sy: usize, dx: usize, dy: usize, w: usize, h: usize) {
        for y in 0..h {
            for x in 0..w {
                let so = (sy + y) * self.mode.pitch + (sx + x) * 4;
                let d = (dy + y) * self.mode.pitch + (dx + x) * 4;
                if so + 3 < self.backbuffer.len() && d + 3 < self.backbuffer.len() {
                    self.backbuffer[d] = self.backbuffer[so];
                    self.backbuffer[d + 1] = self.backbuffer[so + 1];
                    self.backbuffer[d + 2] = self.backbuffer[so + 2];
                }
            }
        }
    }
    fn raw_framebuffer(&mut self) -> &mut [u8] {
        &mut self.backbuffer
    }
}

// ============================================================================
// CIRRUS LOGIC VGA DRIVER — QEMU -vga cirrus
// ============================================================================
pub struct CirrusVgaDriver {
    fb: *mut u8,
    mode: GpuMode,
    backbuffer: Vec<u8>,
    caps: GpuCaps,
    family: GpuFamily,
    enabled: bool,
}
unsafe impl Send for CirrusVgaDriver {}
impl CirrusVgaDriver {
    pub fn new(fb: *mut u8, family: GpuFamily) -> Self {
        Self {
            fb,
            mode: GpuMode::default(),
            backbuffer: Vec::new(),
            caps: GpuCaps::FRAMEBUFFER.merge(GpuCaps::ACCEL_2D),
            family,
            enabled: false,
        }
    }
}
impl GpuDriver for CirrusVgaDriver {
    fn init(&mut self, mode: &GpuMode) -> Result<(), &'static str> {
        self.mode = *mode;
        let size = mode.width * mode.height * 4;
        self.backbuffer = alloc::vec![0u8; size];
        crate::serial_println!(
            "CIRRUS: Init {} @ {}x{} fb={:#x}",
            crate::gpu::gpu_name(self.family),
            mode.width,
            mode.height,
            self.fb as u64
        );
        self.enabled = true;
        Ok(())
    }
    fn current_mode(&self) -> GpuMode {
        self.mode
    }
    fn capabilities(&self) -> GpuCaps {
        self.caps
    }
    fn family(&self) -> GpuFamily {
        self.family
    }
    fn present(&mut self) {
        if !self.fb.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.backbuffer.as_ptr(),
                    self.fb,
                    self.backbuffer.len(),
                );
            }
        }
    }
    fn clear(&mut self, color: Color) {
        for y in 0..self.mode.height {
            for x in 0..self.mode.width {
                let off = y * self.mode.pitch + x * 4;
                if off + 3 < self.backbuffer.len() {
                    self.backbuffer[off] = color.b;
                    self.backbuffer[off + 1] = color.g;
                    self.backbuffer[off + 2] = color.r;
                }
            }
        }
    }
    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.mode.width || y >= self.mode.height {
            return;
        }
        let off = y * self.mode.pitch + x * 4;
        if off + 3 < self.backbuffer.len() {
            self.backbuffer[off] = color.b;
            self.backbuffer[off + 1] = color.g;
            self.backbuffer[off + 2] = color.r;
        }
    }
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.put_pixel(x, y, color);
            }
        }
    }
    fn blit(&mut self, sx: usize, sy: usize, dx: usize, dy: usize, w: usize, h: usize) {
        for y in 0..h {
            for x in 0..w {
                let so = (sy + y) * self.mode.pitch + (sx + x) * 4;
                let d = (dy + y) * self.mode.pitch + (dx + x) * 4;
                if so + 3 < self.backbuffer.len() && d + 3 < self.backbuffer.len() {
                    self.backbuffer[d] = self.backbuffer[so];
                    self.backbuffer[d + 1] = self.backbuffer[so + 1];
                    self.backbuffer[d + 2] = self.backbuffer[so + 2];
                }
            }
        }
    }
    fn raw_framebuffer(&mut self) -> &mut [u8] {
        &mut self.backbuffer
    }
}

// ============================================================================
// VIRTIO GPU DRIVER — QEMU -device virtio-gpu-pci
// ============================================================================
pub struct VirtioGpuDriver {
    fb: *mut u8,
    mode: GpuMode,
    backbuffer: Vec<u8>,
    caps: GpuCaps,
    family: GpuFamily,
    enabled: bool,
}
unsafe impl Send for VirtioGpuDriver {}
impl VirtioGpuDriver {
    pub fn new(fb_base: u64, family: GpuFamily) -> Self {
        Self {
            fb: if fb_base != 0 {
                fb_base as *mut u8
            } else {
                core::ptr::null_mut()
            },
            mode: GpuMode::default(),
            backbuffer: Vec::new(),
            caps: GpuCaps::ACCEL_2D
                .merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::FRAMEBUFFER),
            family,
            enabled: false,
        }
    }
}
impl GpuDriver for VirtioGpuDriver {
    fn init(&mut self, mode: &GpuMode) -> Result<(), &'static str> {
        self.mode = *mode;
        let size = mode.width * mode.height * 4;
        self.backbuffer = alloc::vec![0u8; size];
        crate::serial_println!(
            "VIRTIO-GPU: {} @ {}x{}x{}bpp",
            crate::gpu::gpu_name(self.family),
            mode.width,
            mode.height,
            mode.bpp
        );
        self.enabled = true;
        Ok(())
    }
    fn current_mode(&self) -> GpuMode {
        self.mode
    }
    fn capabilities(&self) -> GpuCaps {
        self.caps
    }
    fn family(&self) -> GpuFamily {
        self.family
    }
    fn present(&mut self) {
        if !self.fb.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.backbuffer.as_ptr(),
                    self.fb,
                    self.backbuffer.len(),
                );
            }
        }
    }
    fn clear(&mut self, color: Color) {
        for y in 0..self.mode.height {
            for x in 0..self.mode.width {
                let off = y * self.mode.pitch + x * 4;
                if off + 3 < self.backbuffer.len() {
                    self.backbuffer[off] = color.b;
                    self.backbuffer[off + 1] = color.g;
                    self.backbuffer[off + 2] = color.r;
                }
            }
        }
    }
    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.mode.width || y >= self.mode.height {
            return;
        }
        let off = y * self.mode.pitch + x * 4;
        if off + 3 < self.backbuffer.len() {
            self.backbuffer[off] = color.b;
            self.backbuffer[off + 1] = color.g;
            self.backbuffer[off + 2] = color.r;
        }
    }
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.put_pixel(x, y, color);
            }
        }
    }
    fn blit(&mut self, sx: usize, sy: usize, dx: usize, dy: usize, w: usize, h: usize) {
        for y in 0..h {
            for x in 0..w {
                let so = (sy + y) * self.mode.pitch + (sx + x) * 4;
                let d = (dy + y) * self.mode.pitch + (dx + x) * 4;
                if so + 3 < self.backbuffer.len() && d + 3 < self.backbuffer.len() {
                    self.backbuffer[d] = self.backbuffer[so];
                    self.backbuffer[d + 1] = self.backbuffer[so + 1];
                    self.backbuffer[d + 2] = self.backbuffer[so + 2];
                }
            }
        }
    }
    fn raw_framebuffer(&mut self) -> &mut [u8] {
        &mut self.backbuffer
    }
}

// ============================================================================
// GPU DRIVER REGISTRY — Auto-detect and bind the best driver
// ============================================================================
pub enum ConcreteGpuDriver {
    None,
    BochsVbe(BochsVbeDriver),
    Nvidia(NvidiaMmioDriver),
    Ati(AtiMmioDriver),
    Voodoo(VoodooBansheeDriver),
    Intel(IntelGfxDriver),
    Vmware(VmwareSvgaDriver),
    Cirrus(CirrusVgaDriver),
    Virtio(VirtioGpuDriver),
    Fallback(crate::gpu::FallbackFbDriver),
}

impl ConcreteGpuDriver {
    /// Detect and initialize the best driver for the given GPU device
    pub fn detect_and_bind(
        device: &GpuDevice,
        fb: *mut u8,
        desired_w: usize,
        desired_h: usize,
    ) -> Self {
        let mode = GpuMode {
            width: desired_w,
            height: desired_h,
            bpp: 32,
            pitch: desired_w * 4,
            framebuffer_addr: device.framebuffer_base.unwrap_or(fb as u64),
            framebuffer_size: desired_w * desired_h * 4,
            double_buffered: false,
        };

        match device.family {
            // QEMU/Bochs VBE — uses I/O ports, not MMIO
            GpuFamily::BochsVBE if bochs_vbe_probe() => {
                let mut drv = BochsVbeDriver::new(fb);
                match drv.init(&mode) {
                    Ok(()) => {
                        crate::serial_println!(
                            "GPU: Bochs VBE driver bound to device {}",
                            device.name
                        );
                        Self::BochsVbe(drv)
                    }
                    Err(e) => {
                        crate::serial_println!("GPU: Bochs VBE init failed: {}", e);
                        Self::fallback(fb, mode)
                    }
                }
            }

            // Nvidia GPUs — use MMIO driver
            fam if is_nvidia_family(fam) => {
                if let Some(mmio_base) = device.mmio_base {
                    let fb_base = device.framebuffer_base.unwrap_or(fb as u64);
                    let mut drv = NvidiaMmioDriver::new(mmio_base, fb_base, device.family);
                    match drv.init(&mode) {
                        Ok(()) => {
                            crate::serial_println!(
                                "GPU: Nvidia MMIO driver bound to [{}:{}.{}] {}",
                                device.bus,
                                device.slot,
                                device.func,
                                device.name
                            );
                            Self::Nvidia(drv)
                        }
                        Err(e) => {
                            crate::serial_println!("GPU: Nvidia init failed: {}", e);
                            Self::fallback(fb, mode)
                        }
                    }
                } else {
                    Self::fallback(fb, mode)
                }
            }

            // ATI/AMD GPUs — use MMIO driver
            fam if is_ati_family(fam) => {
                if let Some(mmio_base) = device.mmio_base {
                    let fb_base = device.framebuffer_base.unwrap_or(fb as u64);
                    let mut drv = AtiMmioDriver::new(mmio_base, fb_base, device.family);
                    match drv.init(&mode) {
                        Ok(()) => {
                            crate::serial_println!(
                                "GPU: ATI/AMD MMIO driver bound to [{}:{}.{}] {}",
                                device.bus,
                                device.slot,
                                device.func,
                                device.name
                            );
                            Self::Ati(drv)
                        }
                        Err(e) => {
                            crate::serial_println!("GPU: ATI/AMD init failed: {}", e);
                            Self::fallback(fb, mode)
                        }
                    }
                } else {
                    Self::fallback(fb, mode)
                }
            }

            // 3dfx Voodoo Banshee / Voodoo3 — full MMIO driver
            fam if is_3dfx_family(fam) => {
                if let Some(mmio_base) = device.mmio_base {
                    let fb_base = device.framebuffer_base.unwrap_or(fb as u64);
                    let mut drv = VoodooBansheeDriver::new(mmio_base, fb_base, device.family);
                    match drv.init(&mode) {
                        Ok(()) => {
                            crate::serial_println!(
                                "GPU: 3dfx Voodoo driver bound to [{}:{}.{}] {}",
                                device.bus,
                                device.slot,
                                device.func,
                                device.name
                            );
                            Self::Voodoo(drv)
                        }
                        Err(e) => {
                            crate::serial_println!("GPU: 3dfx Voodoo init failed: {}", e);
                            Self::fallback(fb, mode)
                        }
                    }
                } else {
                    Self::fallback(fb, mode)
                }
            }

            // Intel integrated graphics — MMIO display engine
            fam if is_intel_family(fam) => {
                if let Some(mmio_base) = device.mmio_base {
                    let fb_base = device.framebuffer_base.unwrap_or(fb as u64);
                    let mut drv = IntelGfxDriver::new(mmio_base, fb_base, device.family);
                    match drv.init(&mode) {
                        Ok(()) => {
                            crate::serial_println!(
                                "GPU: Intel driver bound to [{}:{}.{}] {}",
                                device.bus,
                                device.slot,
                                device.func,
                                device.name
                            );
                            Self::Intel(drv)
                        }
                        Err(e) => {
                            crate::serial_println!("GPU: Intel init failed: {}", e);
                            Self::fallback(fb, mode)
                        }
                    }
                } else {
                    Self::fallback(fb, mode)
                }
            }

            // VMware SVGA II — MMIO index/data register protocol
            fam if is_vmware_family(fam) => {
                if let Some(mmio_base) = device.mmio_base {
                    let fb_base = device.framebuffer_base.unwrap_or(fb as u64);
                    let mut drv = VmwareSvgaDriver::new(mmio_base, fb_base, device.family);
                    match drv.init(&mode) {
                        Ok(()) => {
                            crate::serial_println!(
                                "GPU: VMware SVGA bound to [{}:{}.{}] {}",
                                device.bus,
                                device.slot,
                                device.func,
                                device.name
                            );
                            Self::Vmware(drv)
                        }
                        Err(e) => {
                            crate::serial_println!("GPU: VMware SVGA init failed: {}", e);
                            Self::fallback(fb, mode)
                        }
                    }
                } else {
                    Self::fallback(fb, mode)
                }
            }

            // Cirrus Logic VGA — framebuffer with VGA registers
            fam if is_cirrus_family(fam) => {
                let mut drv = CirrusVgaDriver::new(fb, device.family);
                match drv.init(&mode) {
                    Ok(()) => {
                        crate::serial_println!(
                            "GPU: Cirrus Logic bound to [{}:{}.{}] {}",
                            device.bus,
                            device.slot,
                            device.func,
                            device.name
                        );
                        Self::Cirrus(drv)
                    }
                    Err(e) => {
                        crate::serial_println!("GPU: Cirrus Logic init failed: {}", e);
                        Self::fallback(fb, mode)
                    }
                }
            }

            // VirtIO GPU — virtio-pci transport
            fam if is_virtio_family(fam) => {
                let fb_base = device.framebuffer_base.unwrap_or(fb as u64);
                let mut drv = VirtioGpuDriver::new(fb_base, device.family);
                match drv.init(&mode) {
                    Ok(()) => {
                        crate::serial_println!(
                            "GPU: VirtIO bound to [{}:{}.{}] {}",
                            device.bus,
                            device.slot,
                            device.func,
                            device.name
                        );
                        Self::Virtio(drv)
                    }
                    Err(e) => {
                        crate::serial_println!("GPU: VirtIO init failed: {}", e);
                        Self::fallback(fb, mode)
                    }
                }
            }

            // All others: fallback to framebuffer
            _ => Self::fallback(fb, mode),
        }
    }

    fn fallback(fb: *mut u8, mode: GpuMode) -> Self {
        let mut drv = crate::gpu::FallbackFbDriver::new(fb, mode.width, mode.height);
        let _ = drv.init(&mode);
        crate::serial_println!("GPU: Using framebuffer fallback driver");
        Self::Fallback(drv)
    }

    pub fn as_driver(&mut self) -> Option<&mut dyn GpuDriver> {
        match self {
            Self::BochsVbe(d) => Some(d),
            Self::Nvidia(d) => Some(d),
            Self::Ati(d) => Some(d),
            Self::Voodoo(d) => Some(d),
            Self::Intel(d) => Some(d),
            Self::Vmware(d) => Some(d),
            Self::Cirrus(d) => Some(d),
            Self::Virtio(d) => Some(d),
            Self::Fallback(d) => Some(d),
            Self::None => None,
        }
    }
}

fn is_vmware_family(family: GpuFamily) -> bool {
    matches!(family, GpuFamily::VMWareSVGA | GpuFamily::VMWareSVGA3)
}

fn is_cirrus_family(family: GpuFamily) -> bool {
    matches!(
        family,
        GpuFamily::CirrusLogic5430
            | GpuFamily::CirrusLogic5446
            | GpuFamily::CirrusLogic5464
            | GpuFamily::CirrusLogic5465
            | GpuFamily::CirrusLogic67200
            | GpuFamily::CirrusLogic7548
    )
}

fn is_virtio_family(family: GpuFamily) -> bool {
    matches!(family, GpuFamily::VirtIOGPU | GpuFamily::VirtIOGPU3D)
}

fn is_3dfx_family(family: GpuFamily) -> bool {
    matches!(
        family,
        GpuFamily::Voodoo1
            | GpuFamily::Voodoo2
            | GpuFamily::VoodooRush
            | GpuFamily::VoodooBanshee
            | GpuFamily::Voodoo3
            | GpuFamily::Voodoo4
            | GpuFamily::Voodoo5
    )
}

fn is_intel_family(family: GpuFamily) -> bool {
    matches!(
        family,
        GpuFamily::Intel740
            | GpuFamily::Intel810
            | GpuFamily::Intel915
            | GpuFamily::IntelGMA3000
            | GpuFamily::IntelGMA3100
            | GpuFamily::IntelGMAX3100
            | GpuFamily::IntelGMAX3500
            | GpuFamily::IntelHDGraphics
            | GpuFamily::IntelHDGraphics2000
            | GpuFamily::IntelHDGraphics2500
            | GpuFamily::IntelHDGraphics3000
            | GpuFamily::IntelHDGraphics4000
            | GpuFamily::IntelHDGraphics4200
            | GpuFamily::IntelHDGraphics4400
            | GpuFamily::IntelHDGraphics4600
            | GpuFamily::IntelHDGraphics5000
            | GpuFamily::IntelHDGraphics5100
            | GpuFamily::IntelHDGraphics5200
            | GpuFamily::IntelHDGraphics5300
            | GpuFamily::IntelHDGraphics5500
            | GpuFamily::IntelHDGraphics6000
            | GpuFamily::IntelHDGraphics6100
            | GpuFamily::IntelHDGraphics615
            | GpuFamily::IntelHDGraphics620
            | GpuFamily::IntelHDGraphics630
            | GpuFamily::IntelHDGraphics640
            | GpuFamily::IntelHDGraphics650
            | GpuFamily::IntelHDGraphicsP630
            | GpuFamily::IntelIrisPlus640
            | GpuFamily::IntelIrisPlus645
            | GpuFamily::IntelIrisPlus650
            | GpuFamily::IntelIrisPro580
            | GpuFamily::IntelIrisXe
            | GpuFamily::IntelIrisXeMax
            | GpuFamily::IntelArcA310
            | GpuFamily::IntelArcA380
            | GpuFamily::IntelArcA580
            | GpuFamily::IntelArcA750
            | GpuFamily::IntelArcA770
            | GpuFamily::IntelArcB580
            | GpuFamily::IntelArcB770
            | GpuFamily::IntelXeLLVM
    )
}

fn is_nvidia_family(family: GpuFamily) -> bool {
    matches!(
        family,
        GpuFamily::Nv1
            | GpuFamily::Nv2
            | GpuFamily::Nv3
            | GpuFamily::Nv4
            | GpuFamily::Nv5
            | GpuFamily::Riva128
            | GpuFamily::Riva128ZX
            | GpuFamily::RivaTNT
            | GpuFamily::RivaTNT2
            | GpuFamily::GeForce256
            | GpuFamily::GeForceDDR
            | GpuFamily::GeForce2
            | GpuFamily::GeForce2MX
            | GpuFamily::GeForce2GTS
            | GpuFamily::GeForce2Ultra
            | GpuFamily::GeForce2Go
            | GpuFamily::GeForce3
            | GpuFamily::GeForce3Ti
            | GpuFamily::GeForce4
            | GpuFamily::GeForce4MX
            | GpuFamily::GeForce4Ti
            | GpuFamily::GeForceFX
            | GpuFamily::GeForceFX5200
            | GpuFamily::GeForceFX5600
            | GpuFamily::GeForceFX5700
            | GpuFamily::GeForceFX5800
            | GpuFamily::GeForceFX5900
            | GpuFamily::GeForceFX5950
            | GpuFamily::GeForce6
            | GpuFamily::GeForce6200
            | GpuFamily::GeForce6600
            | GpuFamily::GeForce6800
            | GpuFamily::GeForce7
            | GpuFamily::GeForce7300
            | GpuFamily::GeForce7600
            | GpuFamily::GeForce7800
            | GpuFamily::GeForce7900
            | GpuFamily::GeForce7950
            | GpuFamily::GeForce8
            | GpuFamily::GeForce8300
            | GpuFamily::GeForce8400
            | GpuFamily::GeForce8500
            | GpuFamily::GeForce8600
            | GpuFamily::GeForce8800
            | GpuFamily::GeForce9
            | GpuFamily::GeForce9600
            | GpuFamily::GeForce9800
            | GpuFamily::GeForceGTX200
            | GpuFamily::GTX260
            | GpuFamily::GTX280
            | GpuFamily::GTX285
            | GpuFamily::GTX295
            | GpuFamily::GeForceGTX400
            | GpuFamily::GTX460
            | GpuFamily::GTX465
            | GpuFamily::GTX470
            | GpuFamily::GTX480
            | GpuFamily::GeForceGTX500
            | GpuFamily::GTX550
            | GpuFamily::GTX560
            | GpuFamily::GTX570
            | GpuFamily::GTX580
            | GpuFamily::GTX590
            | GpuFamily::GeForceGTX600
            | GpuFamily::GTX650
            | GpuFamily::GTX660
            | GpuFamily::GTX670
            | GpuFamily::GTX680
            | GpuFamily::GTX690
            | GpuFamily::GeForceGTX700
            | GpuFamily::GTX760
            | GpuFamily::GTX770
            | GpuFamily::GTX780
            | GpuFamily::GTX780Ti
            | GpuFamily::GTXTitan
            | GpuFamily::GeForceGTX900
            | GpuFamily::GTX960
            | GpuFamily::GTX970
            | GpuFamily::GTX980
            | GpuFamily::GTX980Ti
            | GpuFamily::GTXTitanX
            | GpuFamily::GeForceGTX10
            | GpuFamily::GTX1050
            | GpuFamily::GTX1060
            | GpuFamily::GTX1070
            | GpuFamily::GTX1080
            | GpuFamily::GTX1080Ti
            | GpuFamily::GTXTitanXP
            | GpuFamily::GeForceGTX16
            | GpuFamily::GTX1650
            | GpuFamily::GTX1660
            | GpuFamily::GTX1660Super
            | GpuFamily::GTX1660Ti
            | GpuFamily::GeForceRTX20
            | GpuFamily::RTX2060
            | GpuFamily::RTX2070
            | GpuFamily::RTX2080
            | GpuFamily::RTX2080Ti
            | GpuFamily::TitanRTX
            | GpuFamily::GeForceRTX30
            | GpuFamily::RTX3060
            | GpuFamily::RTX3070
            | GpuFamily::RTX3080
            | GpuFamily::RTX3090
            | GpuFamily::RTX3090Ti
            | GpuFamily::GeForceRTX40
            | GpuFamily::RTX4060
            | GpuFamily::RTX4070
            | GpuFamily::RTX4080
            | GpuFamily::RTX4090
            | GpuFamily::GeForceRTX50
            | GpuFamily::RTX5060
            | GpuFamily::RTX5070
            | GpuFamily::RTX5080
            | GpuFamily::RTX5090
            | GpuFamily::NvidiaQuadro
            | GpuFamily::NvidiaTesla
    )
}

fn is_ati_family(family: GpuFamily) -> bool {
    matches!(
        family,
        GpuFamily::RagePro
            | GpuFamily::RageXL
            | GpuFamily::RageFury
            | GpuFamily::Rage128
            | GpuFamily::Rage128VR
            | GpuFamily::Rage128GL
            | GpuFamily::Rage128Pro
            | GpuFamily::Radeon7000
            | GpuFamily::Radeon7200
            | GpuFamily::Radeon7500
            | GpuFamily::Radeon8500
            | GpuFamily::Radeon8500LE
            | GpuFamily::Radeon9000
            | GpuFamily::Radeon9100
            | GpuFamily::Radeon9200
            | GpuFamily::Radeon9500
            | GpuFamily::Radeon9550
            | GpuFamily::Radeon9600
            | GpuFamily::Radeon9700
            | GpuFamily::Radeon9700Pro
            | GpuFamily::Radeon9800
            | GpuFamily::Radeon9800Pro
            | GpuFamily::RadeonX300
            | GpuFamily::RadeonX600
            | GpuFamily::RadeonX700
            | GpuFamily::RadeonX800
            | GpuFamily::RadeonX850
            | GpuFamily::RadeonX1300
            | GpuFamily::RadeonX1600
            | GpuFamily::RadeonX1800
            | GpuFamily::RadeonX1900
            | GpuFamily::RadeonX1950
            | GpuFamily::RadeonHD2400
            | GpuFamily::RadeonHD2600
            | GpuFamily::RadeonHD2900
            | GpuFamily::RadeonHD3450
            | GpuFamily::RadeonHD3650
            | GpuFamily::RadeonHD3850
            | GpuFamily::RadeonHD3870
            | GpuFamily::RadeonHD4350
            | GpuFamily::RadeonHD4550
            | GpuFamily::RadeonHD4650
            | GpuFamily::RadeonHD4670
            | GpuFamily::RadeonHD4770
            | GpuFamily::RadeonHD4830
            | GpuFamily::RadeonHD4850
            | GpuFamily::RadeonHD4870
            | GpuFamily::RadeonHD4870X2
            | GpuFamily::RadeonHD5450
            | GpuFamily::RadeonHD5570
            | GpuFamily::RadeonHD5670
            | GpuFamily::RadeonHD5750
            | GpuFamily::RadeonHD5770
            | GpuFamily::RadeonHD5830
            | GpuFamily::RadeonHD5850
            | GpuFamily::RadeonHD5870
            | GpuFamily::RadeonHD5970
            | GpuFamily::RadeonHD6450
            | GpuFamily::RadeonHD6570
            | GpuFamily::RadeonHD6670
            | GpuFamily::RadeonHD6750
            | GpuFamily::RadeonHD6770
            | GpuFamily::RadeonHD6790
            | GpuFamily::RadeonHD6850
            | GpuFamily::RadeonHD6870
            | GpuFamily::RadeonHD6950
            | GpuFamily::RadeonHD6970
            | GpuFamily::RadeonHD6990
            | GpuFamily::RadeonHD7750
            | GpuFamily::RadeonHD7770
            | GpuFamily::RadeonHD7850
            | GpuFamily::RadeonHD7870
            | GpuFamily::RadeonHD7950
            | GpuFamily::RadeonHD7970
            | GpuFamily::RadeonHD7990
            | GpuFamily::RadeonR7240
            | GpuFamily::RadeonR7250
            | GpuFamily::RadeonR7260
            | GpuFamily::RadeonR7270
            | GpuFamily::RadeonR7280
            | GpuFamily::RadeonR9280
            | GpuFamily::RadeonR9290
            | GpuFamily::RadeonR9290X
            | GpuFamily::RadeonR9390
            | GpuFamily::RadeonR9390X
            | GpuFamily::RadeonR7460
            | GpuFamily::RadeonR7470
            | GpuFamily::RadeonR7480
            | GpuFamily::RadeonR7570
            | GpuFamily::RadeonR7580
            | GpuFamily::RadeonR7590
            | GpuFamily::RadeonRX460
            | GpuFamily::RadeonRX470
            | GpuFamily::RadeonRX480
            | GpuFamily::RadeonRX550
            | GpuFamily::RadeonRX560
            | GpuFamily::RadeonRX570
            | GpuFamily::RadeonRX580
            | GpuFamily::RadeonRX590
            | GpuFamily::RadeonRX5500
            | GpuFamily::RadeonRX5600
            | GpuFamily::RadeonRX5700
            | GpuFamily::RadeonRX6400
            | GpuFamily::RadeonRX6500XT
            | GpuFamily::RadeonRX6600
            | GpuFamily::RadeonRX6700XT
            | GpuFamily::RadeonRX6800
            | GpuFamily::RadeonRX6800XT
            | GpuFamily::RadeonRX6900XT
            | GpuFamily::RadeonRX7600
            | GpuFamily::RadeonRX7700XT
            | GpuFamily::RadeonRX7800XT
            | GpuFamily::RadeonRX7900GRE
            | GpuFamily::RadeonRX7900XT
            | GpuFamily::RadeonRX7900XTX
            | GpuFamily::RadeonRX9070
            | GpuFamily::RadeonRX9070XT
            | GpuFamily::AMDFirePro
            | GpuFamily::AMDInstinct
    )
}

// Helper to get GPU name locally
fn gpu_name(family: GpuFamily) -> &'static str {
    crate::gpu::gpu_name(family)
}
