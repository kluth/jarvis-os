use crate::sync::Spinlock;
use core::fmt;
use x86_64::instructions::port::Port;

pub struct SerialPort {
    data: Port<u8>,
    int_en: Port<u8>,
    fifo_ctrl: Port<u8>,
    line_ctrl: Port<u8>,
    modem_ctrl: Port<u8>,
    line_sts: Port<u8>,
}

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self {
            data: Port::new(port),
            int_en: Port::new(port + 1),
            fifo_ctrl: Port::new(port + 2),
            line_ctrl: Port::new(port + 3),
            modem_ctrl: Port::new(port + 4),
            line_sts: Port::new(port + 5),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.int_en.write(0x00);
            self.line_ctrl.write(0x80);
            self.data.write(0x03);
            self.int_en.write(0x00);
            self.line_ctrl.write(0x03);
            self.fifo_ctrl.write(0xC7);
            self.modem_ctrl.write(0x0B);
            self.int_en.write(0x01);
        }
    }

    fn wait_for_tx_empty(&mut self) {
        unsafe {
            while (self.line_sts.read() & 0x20) == 0 {
                core::hint::spin_loop();
            }
        }
    }

    pub fn send(&mut self, data: u8) {
        match data {
            8 | 0x7F => {
                self.wait_for_tx_empty();
                unsafe { self.data.write(8) };
                self.wait_for_tx_empty();
                unsafe { self.data.write(b' ') };
                self.wait_for_tx_empty();
                unsafe { self.data.write(8) };
            }
            _ => {
                self.wait_for_tx_empty();
                unsafe { self.data.write(data) };
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

pub static SERIAL1: Spinlock<SerialPort> = Spinlock::new(SerialPort::new(0x3F8));

pub fn init() {
    SERIAL1.lock().init();
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! serial_println_raw {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
