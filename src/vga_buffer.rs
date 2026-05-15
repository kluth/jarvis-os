use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use core::{fmt, ptr};
use font8x8::UnicodeFonts;
use lazy_static::lazy_static;
use spinning_top::Spinlock;

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

    pub fn clear(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        for i in 0..self.framebuffer.len() {
            self.framebuffer[i] = 0;
        }
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
                                // Cyan text for JARVIS
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
            ptr::copy(
                self.framebuffer.as_ptr().add(bytes_to_scroll),
                self.framebuffer.as_mut_ptr(),
                total_bytes - bytes_to_scroll,
            );
        }

        // Clear the last line
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

/// Initializes the global writer.
pub fn init(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();
    let writer = FramebufferWriter::new(framebuffer.buffer_mut(), info);
    *WRITER.lock() = Some(writer);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    if let Some(writer) = WRITER.lock().as_mut() {
        writer.write_fmt(args).unwrap();
    }
}
