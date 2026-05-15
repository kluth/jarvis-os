#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

// Import library components
use jarvis_kernel::{acpi, gdt, gui, interrupts, memory, serial_println, telemetry, vga_buffer};

#[cfg(feature = "test")]
use jarvis_kernel::qemu;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("PANIC: {}", info);

    #[cfg(feature = "test")]
    qemu::exit_qemu(qemu::QemuExitCode::Failed);

    #[cfg(not(feature = "test"))]
    loop {
        core::hint::spin_loop();
    }
}

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory =
        core::option::Option::Some(bootloader_api::config::Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use jarvis_kernel::ai;
    use jarvis_kernel::allocator;
    use jarvis_kernel::audio;
    use jarvis_kernel::net;
    use jarvis_kernel::pci;
    use jarvis_kernel::storage;
    use jarvis_kernel::task::{executor::Executor, Task};

    // 1. Initialize Framebuffer as early as possible
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        vga_buffer::init(framebuffer);
    }

    // 2. Initialize UI
    gui::init_ui();

    serial_println!("Hello JARVIS OS!");

    // 3. Initialize CPU & Memory infrastructure
    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("Physical memory offset not provided by bootloader"),
    );

    serial_println!("Initializing CPU features...");
    gdt::init();
    interrupts::init_idt();

    serial_println!("Initializing Memory Mapper...");
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    serial_println!("Initializing Bitmap Frame Allocator...");
    let mut frame_allocator =
        unsafe { memory::BitmapFrameAllocator::init(&boot_info.memory_regions, phys_mem_offset) };

    // 4. Initialize Heap (CRITICAL: must be before any telemetry or complex logging)
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    serial_println!("Status: Memory management initialized.");

    // 5. Initialize APIC and Interrupts
    serial_println!("Initializing APIC...");
    unsafe { interrupts::init_apic(phys_mem_offset) };
    x86_64::instructions::interrupts::enable();

    // Now we can use telemetry and other heap-dependent systems
    telemetry::log(telemetry::TelemetryData::SystemStatus("Booting..."));

    if let Some(rsdp_addr) = boot_info.rsdp_addr.into_option() {
        acpi::init(x86_64::PhysAddr::new(rsdp_addr));
    }

    #[cfg(feature = "test")]
    run_tests();

    serial_println!("Scanning PCI bus...");
    let hda_devices = pci::scan_bus();
    for dev in hda_devices {
        serial_println!(
            "Found HDA at {}:{}:{} (BAR0: 0x{:x})",
            dev.bus,
            dev.slot,
            dev.function,
            dev.read_bar(0)
        );

        let mut controller = unsafe { audio::hda::HdaController::new(&dev, phys_mem_offset) };
        unsafe {
            controller.init();
        }
    }

    // Initialize networking after PCI scan
    net::init();

    let mut jfs = storage::jfs::Jfs::new(1024 * 64); // 64 KiB RamDisk
    {
        use jarvis_kernel::storage::vfs::FileSystem;
        let mut _file = jfs.create("audio_log.raw").expect("Failed to create file");
        // let _ = _file.write(b"JARVIS Audio Data Placeholder");
    }

    let mut executor = Executor::new();
    executor.spawn(Task::with_priority(
        ai::vad_task(1000),
        jarvis_kernel::task::Priority::High,
    ));
    executor.spawn(Task::new(ai::shell::shell_task()));
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
    jarvis_kernel::serial_println!("[ok]");
}
