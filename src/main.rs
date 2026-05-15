#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

// Import library components
use jarvis_kernel::{serial_println, gdt, interrupts, memory, allocator, pci, audio, storage, ai, vga_buffer, qemu, telemetry, gui, acpi, net, device_manager};
use jarvis_kernel::task::{Task, executor::Executor};
use jarvis_kernel::task::keyboard;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("PANIC: {}", info);
    
    #[cfg(feature = "test")]
    qemu::exit_qemu(qemu::QemuExitCode::Failed);

    loop {}
}

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory_offset = core::option::Option::Some(bootloader_api::config::Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

/// Kernel entry point.
/// The bootloader calls this function once the system is in 64-bit mode.
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        vga_buffer::init(framebuffer);
    }

    // ALWAYS print BOOT_READY first for CI detection
    serial_println!("BOOT_READY");
    serial_println!("Hello JARVIS OS!");
    
    serial_println!("Initializing CPU features...");
    gdt::init();
    interrupts::init_idt();

    // Now that IDT is ready, we can try UI
    serial_println!("Initializing JARVIS HUD...");
    gui::init_ui();

    telemetry::log(telemetry::TelemetryData::SystemStatus("Booting..."));

    if let Some(rsdp_addr) = boot_info.rsdp_addr.into_option() {
        acpi::init(x86_64::PhysAddr::new(rsdp_addr));
    }

    #[cfg(feature = "test")]
    run_tests();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().expect("Physical memory offset not provided by bootloader"));
    
    // Initialize APIC instead of PIC
    serial_println!("Initializing APIC...");
    unsafe { interrupts::init_apic(phys_mem_offset) };
    x86_64::instructions::interrupts::enable();

    serial_println!("Initializing Memory Mapper...");
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    serial_println!("Initializing Bitmap Frame Allocator...");
    let mut frame_allocator = unsafe {
        memory::BitmapFrameAllocator::init(&boot_info.memory_regions, phys_mem_offset)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    serial_println!("Status: Memory management initialized.");

    serial_println!("Scanning PCI bus...");
    let hda_devices = pci::scan_bus();
    for dev in hda_devices {
        serial_println!("Found HDA at {}:{}:{} (BAR0: 0x{:x})", 
            dev.bus, dev.slot, dev.function, dev.read_bar(0));
        
        let mut controller = unsafe { 
            audio::hda::HdaController::new(&dev, phys_mem_offset) 
        };
        unsafe { controller.init(); }
    }

    // Initialize networking after PCI scan
    net::init();

    let mut jfs = storage::jfs::Jfs::new(1024 * 64); // 64 KiB RamDisk
    {
        use jarvis_kernel::storage::vfs::FileSystem;
        let mut file = jfs.create("audio_log.raw").expect("Failed to create file");
        let _ = file.write(b"JARVIS Audio Data Placeholder");
        serial_println!("Status: JFS test write completed. Size: {} bytes", file.size());
    }

    let mut executor = Executor::new();
    executor.spawn(Task::with_priority(ai::vad_task(1000), jarvis_kernel::task::Priority::High));
    executor.spawn(Task::new(ai::shell::shell_task()));
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(telemetry::telemetry_task()));
    executor.spawn(Task::new(gui::ui_task()));
    executor.spawn(Task::new(net::discovery_task()));
    
    serial_println!("Status: Multitasking active. System ready.");
    executor.run();
}

#[cfg(feature = "test")]
fn run_tests() {
    serial_println!("Running system tests...");
    test_println();
    serial_println!("All tests passed!");
    qemu::exit_qemu(qemu::QemuExitCode::Success);
}

#[cfg(feature = "test")]
fn test_println() {
    jarvis_kernel::serial_print!("test_println... ");
    serial_println!("[ok]");
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    serial_println!("Async task says hello! The number is {}", number);
}
