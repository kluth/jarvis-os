use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use core::fmt;
use font8x8::UnicodeFonts;
use lazy_static::lazy_static;
use spinning_top::Spinlock;
use x86_64::VirtAddr;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// A simple writer for the UEFI Framebuffer.
pub struct FramebufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl FramebufferWriter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut writer = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        writer.clear();
        writer
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }
        let pixel_offset = y * self.info.stride + x;
        let c = match self.info.pixel_format {
            PixelFormat::Rgb => [color.r, color.g, color.b, 0],
            PixelFormat::Bgr => [color.b, color.g, color.r, 0],
            PixelFormat::U8 => [
                ((color.r as u16 + color.g as u16 + color.b as u16) / 3) as u8,
                0,
                0,
                0,
            ],
            _ => [color.r, color.g, color.b, 0],
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&c[..bytes_per_pixel]);
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        for i in 0..rect.width {
            self.write_pixel(rect.x + i, rect.y, color);
            self.write_pixel(rect.x + i, rect.y + rect.height - 1, color);
        }
        for i in 0..rect.height {
            self.write_pixel(rect.x, rect.y + i, color);
            self.write_pixel(rect.x + rect.width - 1, rect.y + i, color);
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        for i in 0..rect.width {
            for j in 0..rect.height {
                self.write_pixel(rect.x + i, rect.y + j, color);
            }
        }
    }

    /// Bresenham's line algorithm
    pub fn draw_line(&mut self, x0: isize, y0: isize, x1: isize, y1: isize, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && y >= 0 {
                self.write_pixel(x as usize, y as usize, color);
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn clear(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        self.framebuffer.fill(0);
    }

    /// Writes a single character using font8x8.
    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            _ => {
                if let Some(glyph) = font8x8::BASIC_FONTS.get(c) {
                    for (y, byte) in glyph.iter().enumerate() {
                        for x in 0..8 {
                            if (byte & (1 << x)) != 0 {
                                self.write_pixel(
                                    self.x_pos + x,
                                    self.y_pos + y,
                                    Color {
                                        r: 0,
                                        g: 255,
                                        b: 255,
                                    },
                                );
                            }
                        }
                    }
                }
                self.x_pos += 8;
                if self.x_pos >= self.info.width {
                    self.newline();
                }
            }
        }
    }

    pub fn write_char_at(&mut self, x: usize, y: usize, c: char, color: Color) {
        if let Some(glyph) = font8x8::BASIC_FONTS.get(c) {
            for (dy, byte) in glyph.iter().enumerate() {
                for dx in 0..8 {
                    if (byte & (1 << dx)) != 0 {
                        self.write_pixel(x + dx, y + dy, color);
                    }
                }
            }
        }
    }

    pub fn write_string_at(&mut self, x: usize, y: usize, s: &str, color: Color) {
        let mut curr_x = x;
        for c in s.chars() {
            self.write_char_at(curr_x, y, c, color);
            curr_x += 8;
        }
    }

    pub fn get_info(&self) -> FrameBufferInfo {
        self.info
    }

    pub fn raw_bytes(&self) -> &[u8] {
        self.framebuffer
    }

    pub fn raw_bytes_mut(&mut self) -> &mut [u8] {
        self.framebuffer
    }

    fn newline(&mut self) {
        self.y_pos += 16;
        self.x_pos = 0;
        if self.y_pos >= self.info.height {
            self.scroll();
        }
    }

    fn scroll(&mut self) {
        let bytes_per_line = self.info.stride * self.info.bytes_per_pixel;
        let bytes_to_scroll = 16 * bytes_per_line;
        let total_bytes = self.info.height * bytes_per_line;

        unsafe {
            core::ptr::copy(
                self.framebuffer.as_ptr().add(bytes_to_scroll),
                self.framebuffer.as_mut_ptr(),
                total_bytes - bytes_to_scroll,
            );
        }

        let last_line_start = total_bytes - bytes_to_scroll;
        for i in last_line_start..total_bytes {
            self.framebuffer[i] = 0;
        }

        self.y_pos -= 16;
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }
}

impl fmt::Write for FramebufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Spinlock<Option<FramebufferWriter>> = Spinlock::new(None);
}

static mut FRAMEBUFFER_SAVED_INFO: Option<FrameBufferInfo> = None;

pub fn init(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();
    let writer = FramebufferWriter::new(framebuffer.buffer_mut(), info);
    *WRITER.lock() = Some(writer);
    unsafe {
        FRAMEBUFFER_SAVED_INFO = Some(info);
    }
}

/// Re-initialize framebuffer at `phys_mem_offset + fb_phys_addr` using non-temporal stores
pub unsafe fn reinit_via_phys_mem_offset(phys_mem_offset: u64, fb_phys_addr: u64) {
    if let Some(info) = FRAMEBUFFER_SAVED_INFO {
        let bytes_per_line = info.stride * info.bytes_per_pixel;
        let byte_len = info.height * bytes_per_line;
        let fb_virt = VirtAddr::new(phys_mem_offset + fb_phys_addr);
        let fb_slice = core::slice::from_raw_parts_mut(fb_virt.as_mut_ptr::<u8>(), byte_len);
        let writer = FramebufferWriter {
            framebuffer: fb_slice,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        *WRITER.lock() = Some(writer);
        crate::serial_println!(
            "VGA: remapped @ phys+0x{:x}=VA 0x{:x}",
            fb_phys_addr,
            phys_mem_offset + fb_phys_addr
        );
    }
}

/// Flush framebuffer cache using WBINVD
pub fn flush_fb() {
    unsafe {
        core::arch::asm!("wbinvd");
    }
}

/// Create a FramebufferWriter at a specific virtual address (after manual page table mapping).
///
/// This is called after `init_heap()` when we've created fresh page table entries for the
/// framebuffer (at `va`) to avoid bootloader page table corruption. The physical backing is
/// assumed to be the same as the original framebuffer — only the virtual address changes.
///
/// # Safety
/// Caller must ensure `va` points to a valid framebuffer mapping with the correct physical
/// backing, size >= `height * stride * bytes_per_pixel`.
pub unsafe fn create_writer_at(va: VirtAddr, _buf_size: usize) {
    if let Some(info) = FRAMEBUFFER_SAVED_INFO {
        let bytes_per_line = info.stride * info.bytes_per_pixel;
        let byte_len = info.height * bytes_per_line;
        let fb_slice = core::slice::from_raw_parts_mut(va.as_mut_ptr::<u8>(), byte_len);
        let writer = FramebufferWriter {
            framebuffer: fb_slice,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        *WRITER.lock() = Some(writer);
        crate::serial_println!(
            "VGA: created writer at VA 0x{:x} ({} bytes)",
            va.as_u64(),
            byte_len
        );
    } else {
        crate::serial_println!(
            "VGA ERROR: create_writer_at called but FRAMEBUFFER_SAVED_INFO is None"
        );
    }
}

/// Write to framebuffer using non-temporal stores (bypass cache)
pub fn write_nt(phys_offset: u64, data: &[u8]) {
    unsafe {
        let dst = phys_offset as *mut u8;
        for i in (0..data.len()).step_by(8) {
            let chunk = &data[i..core::cmp::min(i + 8, data.len())];
            if chunk.len() == 8 {
                let val = u64::from_ne_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                ]);
                core::arch::asm!("movnti [{addr}], {val}",
                    addr = in(reg) dst.add(i),
                    val = in(reg) val);
            } else if chunk.len() >= 4 {
                let val = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                core::arch::asm!("movnti [{addr}], {val}",
                    addr = in(reg) dst.add(i),
                    val = in(reg) val);
            }
        }
        core::arch::asm!("sfence");
    }
}
