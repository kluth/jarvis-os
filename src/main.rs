#![no_std]
#![no_main]

mod vga_buffer;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

/// Der Panic-Handler wird aufgerufen, wenn im Kernel ein fataler Fehler auftritt.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

entry_point!(kernel_main);

/// Der Einstiegspunkt für den Kernel.
/// Der Bootloader ruft diese Funktion auf, sobald das System im 64-Bit Modus ist.
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        vga_buffer::init(framebuffer);
    }

    println!("Hello JARVIS OS!");
    println!("Status: Bootloader Phase 1 completed.");
    println!("System ready for further initialization...");

    loop {}
}

