#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(jarvis_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use jarvis_kernel::{serial_print, serial_println};

entry_point!(main);

fn main(_boot_info: &'static mut BootInfo) -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    jarvis_kernel::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    serial_print!("test_println... ");
    serial_println!("[ok]");
}
