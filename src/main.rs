#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod vga_buffer;
mod gdt;
mod interrupts;

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
    
    init();

    println!("Status: CPU Phase 1 completed (GDT/IDT initialized).");
    
    // Trigger a breakpoint exception to verify IDT
    x86_64::instructions::interrupts::int3();

    println!("It did not crash! Breakpoint handled.");

    loop {}
}

fn init() {
    gdt::init();
    interrupts::init_idt();
}

