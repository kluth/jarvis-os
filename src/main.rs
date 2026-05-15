#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

// Import library components
use jarvis_kernel::{acpi, gdt, interrupts, memory, serial_println};

#[cfg(feature = "gui")]
use jarvis_kernel::{gui, vga_buffer};

#[cfg(feature = "telemetry")]
use jarvis_kernel::telemetry;

#[cfg(feature = "test")]
use jarvis_kernel::{qemu, serial_print};

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
    use jarvis_kernel::allocator;
    use jarvis_kernel::pci;
    use jarvis_kernel::task::executor::Executor;

    #[cfg(feature = "ai")]
    use jarvis_kernel::ai;
    #[cfg(feature = "audio")]
    use jarvis_kernel::audio;
    #[cfg(feature = "network")]
    use jarvis_kernel::net;
    #[cfg(feature = "storage")]
    use jarvis_kernel::storage;

    #[cfg(any(
        feature = "ai",
        feature = "telemetry",
        feature = "gui",
        feature = "network"
    ))]
    use jarvis_kernel::task::Task;

    serial_println!("Hello JARVIS OS!");

    // 1. Initialize Framebuffer as early as possible
    #[cfg(feature = "gui")]
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        vga_buffer::init(framebuffer);
    }

    // 2. Initialize CPU & Memory infrastructure
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

    // 3. Initialize Heap
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    serial_println!("Status: Memory management initialized.");

    // 4. Initialize APIC and Interrupts
    serial_println!("Initializing APIC...");
    unsafe { interrupts::init_apic(phys_mem_offset) };
    x86_64::instructions::interrupts::enable();

    // 5. Initialize Subsystems
    #[cfg(feature = "telemetry")]
    telemetry::log(telemetry::TelemetryData::SystemStatus("Booting..."));

    if let Some(rsdp_addr) = boot_info.rsdp_addr.into_option() {
        acpi::init(x86_64::PhysAddr::new(rsdp_addr));
    }

    serial_println!("Scanning PCI bus...");
    let hda_devices = pci::scan_bus();

    #[cfg(feature = "test")]
    run_tests();
    for dev in hda_devices {
        serial_println!(
            "Found HDA at {}:{}:{} (BAR0: 0x{:x})",
            dev.bus,
            dev.slot,
            dev.function,
            dev.read_bar(0)
        );

        #[cfg(feature = "audio")]
        {
            let mut controller = unsafe { audio::hda::HdaController::new(&dev, phys_mem_offset) };
            unsafe {
                controller.init();
            }
        }
    }

    #[cfg(feature = "network")]
    net::init();

    #[cfg(feature = "storage")]
    {
        let mut jfs = storage::jfs::Jfs::new(1024 * 64); // 64 KiB RamDisk
        use jarvis_kernel::storage::vfs::FileSystem;
        let mut _file = jfs.create("audio_log.raw").expect("Failed to create file");
    }

    // 6. Start Multitasking
    let mut executor = Executor::new();

    #[cfg(feature = "ai")]
    {
        executor.spawn(Task::with_priority(
            ai::vad_task(1000),
            jarvis_kernel::task::Priority::High,
        ));
        executor.spawn(Task::new(ai::shell::shell_task()));
    }

    #[cfg(feature = "telemetry")]
    executor.spawn(Task::new(telemetry::telemetry_task()));

    #[cfg(feature = "gui")]
    executor.spawn(Task::new(gui::ui_task()));

    #[cfg(feature = "network")]
    executor.spawn(Task::new(net::discovery_task()));

    // 7. Initialize UI (last, just before yielding control)
    #[cfg(feature = "gui")]
    gui::init_ui();

    serial_println!("Status: Multitasking active. System ready.");
    executor.run();
}

#[cfg(feature = "test")]
fn run_tests() {
    serial_println!("Running system tests...");
    test_println();
    test_pci_discovery();
    serial_println!("All tests passed!");
    qemu::exit_qemu(qemu::QemuExitCode::Success);
}

#[cfg(feature = "test")]
fn test_pci_discovery() {
    serial_print!("test_pci_discovery... ");
    // Ensure PCI scan has run (it runs in kernel_main)
    let devices = jarvis_kernel::device_manager::MANAGER.lock();
    assert!(
        !devices.get_devices().is_empty(),
        "No devices registered in Device Manager"
    );
    serial_println!("[ok] (found {} devices)", devices.get_devices().len());
}

#[cfg(feature = "test")]
fn test_println() {
    jarvis_kernel::serial_print!("test_println... ");
    jarvis_kernel::serial_println!("[ok]");
}
