use bootloader::boot_info::{Framebuffer, FramebufferConfig, PixelFormat};
use core::{fmt, ptr};
use spinning_top::Spinlock;
use lazy_static::lazy_static;

/// Ein einfacher Writer für den UEFI Framebuffer.
pub struct FramebufferWriter {
    framebuffer: &'static mut [u8],
    config: FramebufferConfig,
    x_pos: usize,
    y_pos: usize,
}

impl FramebufferWriter {
    pub fn new(framebuffer: &'static mut [u8], config: FramebufferConfig) -> Self {
        let mut writer = Self {
            framebuffer,
            config,
            x_pos: 0,
            y_pos: 0,
        };
        writer.clear();
        writer
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.config.stride + x;
        let color = match self.config.pixel_format {
            PixelFormat::Rgb => [intensity, intensity, intensity, 0],
            PixelFormat::Bgr => [intensity, intensity, intensity, 0],
            PixelFormat::U8 => [intensity, 0, 0, 0],
            _ => [intensity, intensity, intensity, 0],
        };
        let bytes_per_pixel = self.config.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
    }

    pub fn clear(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        self.framebuffer.fill(0);
    }

    /// Schreibt ein einzelnes Zeichen unter Verwendung von font8x8.
    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            _ => {
                if let Some(glyph) = font8x8::BASIC_FONTS.get(c as usize) {
                    for (y, byte) in glyph.iter().enumerate() {
                        for x in 0..8 {
                            if (byte & (1 << x)) != 0 {
                                self.write_pixel(self.x_pos + x, self.y_pos + y, 255);
                            }
                        }
                    }
                }
                self.x_pos += 8;
                if self.x_pos >= self.config.width {
                    self.newline();
                }
            }
        }
    }

    fn newline(&mut self) {
        self.y_pos += 16;
        self.x_pos = 0;
        if self.y_pos >= self.config.height {
            self.scroll();
        }
    }

    fn scroll(&mut self) {
        let bytes_per_line = self.config.stride * self.config.bytes_per_pixel;
        let bytes_to_scroll = 16 * bytes_per_line;
        let total_bytes = self.config.height * bytes_per_line;

        unsafe {
            ptr::copy(
                self.framebuffer.as_ptr().add(bytes_to_scroll),
                self.framebuffer.as_mut_ptr(),
                total_bytes - bytes_to_scroll,
            );
        }

        // Letzte Zeile löschen
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

/// Initialisiert den globalen Writer.
pub fn init(framebuffer: &'static mut Framebuffer) {
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
