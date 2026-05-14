#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod vga_buffer;
mod gdt;
mod interrupts;
mod memory;
mod allocator;
mod serial;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

/// Der Panic-Handler wird aufgerufen, wenn im Kernel ein fataler Fehler auftritt.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    serial_println!("{}", info);
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

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    println!("Status: Memory management initialized (Heap active).");

    use alloc::boxed::Box;
    let x = Box::new(42);
    println!("Heap test: Box value is {}", x);

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

