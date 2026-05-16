use lazy_static::lazy_static;
use spinning_top::Spinlock;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Spinlock<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Spinlock::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // 1. Print to Serial (Always) and VGA (if enabled)
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");

        #[cfg(feature = "gui")]
        {
            if let Some(writer) = crate::vga_buffer::WRITER.lock().as_mut() {
                writer.write_fmt(args).unwrap();
            }
        }
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

/// Standard print macro, maps to hardware-aware serial_print.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::serial_print!($($arg)*));
}

/// Standard println macro, maps to hardware-aware serial_println.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
