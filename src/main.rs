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
mod task;
mod apic;
mod pci;
mod audio;
mod storage;
mod ai;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;
use crate::task::{Task, executor::Executor};
use crate::task::keyboard;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    serial_println!("{}", info);
    loop {}
}

entry_point!(kernel_main);

/// Kernel entry point.
/// The bootloader calls this function once the system is in 64-bit mode.
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        vga_buffer::init(framebuffer);
    }

    println!("Hello JARVIS OS!");
    serial_println!("BOOT_READY");
    
    gdt::init();
    interrupts::init_idt();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    
    // Initialize APIC instead of PIC
    unsafe { interrupts::init_apic(phys_mem_offset) };
    x86_64::instructions::interrupts::enable();

    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BitmapFrameAllocator::init(&boot_info.memory_regions, phys_mem_offset)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    println!("Status: Memory management initialized.");

    let hda_devices = pci::scan_bus();
    for dev in hda_devices {
        println!("Found HDA at {}:{}:{} (BAR0: 0x{:x})", 
            dev.bus, dev.slot, dev.function, dev.read_bar(0));
        
        let mut controller = unsafe { 
            audio::hda::HdaController::new(&dev, phys_mem_offset) 
        };
        unsafe { controller.init(); }
    }

    let mut jfs = storage::jfs::Jfs::new(1024 * 64); // 64 KiB RamDisk
    {
        use crate::storage::vfs::FileSystem;
        let mut file = jfs.create("audio_log.raw").expect("Failed to create file");
        file.write(b"JARVIS Audio Data Placeholder").expect("Failed to write to file");
        println!("Status: JFS test write completed. Size: {} bytes", file.size());
    }

    let mut executor = Executor::new();
    executor.spawn(Task::with_priority(ai::vad_task(1000), crate::task::Priority::High));
    executor.spawn(Task::new(ai::shell::shell_task()));
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    
    println!("Status: Multitasking active. System ready.");
    executor.run();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("Async task says hello! The number is {}", number);
}
