use lazy_static::lazy_static;
use spinning_top::Spinlock;
use uart_16550::SerialPort;

pub const COM1_PORT: u16 = 0x3F8;

lazy_static! {
    pub static ref SERIAL1: Spinlock<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(COM1_PORT) };
        serial_port.init();
        Spinlock::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // 1. Print to Serial (Always)
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });

    // 2. Push to System Log for UI (if allocation is safe and heap is ready)
    #[cfg(feature = "gui")]
    {
        use alloc::string::ToString;
        // Safety: Only log if we are not in an early boot phase where heap might be flaky.
        // We check a global flag that is set after heap init.
        if crate::is_heap_ready() && args.as_str().map(|s| s.len()).unwrap_or(0) < 64 {
            let s = args.to_string();
            crate::telemetry::push_log(s);
        }
    }
}

/// A raw, lock-free serial write for use in exceptions and ISRs.
///
/// # Safety
///
/// This bypasses all locks. Concurrent writes from other CPUs or tasks
/// may cause interleaved output. Only use for critical diagnostics.
pub unsafe fn write_str_raw(s: &str) {
    let mut serial_port = SerialPort::new(COM1_PORT);
    for byte in s.bytes() {
        serial_port.send(byte);
    }
}

pub struct RawSerialWriter;

impl core::fmt::Write for RawSerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            write_str_raw(s);
        }
        Ok(())
    }
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

#[doc(hidden)]
pub fn _print_raw(args: core::fmt::Arguments) {
    use core::fmt::Write;
    let _ = RawSerialWriter.write_fmt(args);
}

/// A raw, lock-free println macro for use in exceptions and ISRs.
#[macro_export]
macro_rules! serial_println_raw {
    ($($arg:tt)*) => {
        $crate::serial::_print_raw(format_args!($($arg)*));
        $crate::serial::_print_raw(format_args!("\n"));
    };
}
