#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use jarvis_kernel::task::executor::Executor;
use jarvis_kernel::task::Task;
use jarvis_kernel::{gui, memory, net, serial_println, telemetry};
use x86_64::{PhysAddr, VirtAddr};

pub const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // 1. Initialize hardware SSE support immediately
    unsafe {
        jarvis_kernel::enable_sse();
    }

    // 2. Initialize Core Subsystems
    jarvis_kernel::gdt::init();
    jarvis_kernel::interrupts::init_idt();

    let phys_mem_offset_raw = boot_info.physical_memory_offset.into_option().unwrap();
    let phys_mem_offset = VirtAddr::new(phys_mem_offset_raw);

    unsafe {
        let mut apic = jarvis_kernel::apic::LocalApic::new(phys_mem_offset);
        apic.init();
        jarvis_kernel::apic::disable_pic();
    };

    // 3. Initialize Memory Management
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BitmapFrameAllocator::init(&boot_info.memory_regions, phys_mem_offset) };

    jarvis_kernel::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    serial_println!("Status: Memory management initialized.");

    // 4. Initialize Device Discovery
    // Note: In QEMU/BIOS, RSDP is often at 0xf52b0
    jarvis_kernel::acpi::init(PhysAddr::new(0xf52b0));
    jarvis_kernel::pci::scan_bus();

    // 5. Initialize Services
    #[cfg(feature = "storage")]
    {
        use jarvis_kernel::storage;
        let mut jfs = storage::jfs::Jfs::new(1024 * 64); // 64 KiB RamDisk
        use jarvis_kernel::storage::vfs::FileSystem;
        let mut _file = jfs.create("audio_log.raw").expect("Failed to create file");
    }

    // 6. Start Multitasking
    let mut executor = Executor::new();

    #[cfg(feature = "storage")]
    executor.spawn(Task::new(jarvis_kernel::storage::dht::dht_task()));

    #[cfg(feature = "ai")]
    {
        use jarvis_kernel::ai;
        executor.spawn(Task::with_priority(
            ai::vad_task(1000),
            jarvis_kernel::task::Priority::High,
        ));
        executor.spawn(Task::new(ai::shell::shell_task()));
        executor.spawn(Task::new(ai::swarm::swarm_task()));
    }

    #[cfg(feature = "telemetry")]
    {
        executor.spawn(Task::new(telemetry::telemetry_task()));
        executor.spawn(Task::new(
            jarvis_kernel::sensors::biometrics::biometrics_task(),
        ));
    }

    #[cfg(feature = "gui")]
    executor.spawn(Task::new(gui::ui_task()));

    executor.spawn(Task::new(jarvis_kernel::security::security_task()));

    #[cfg(feature = "network")]
    {
        executor.spawn(Task::new(net::discovery_task()));
        executor.spawn(Task::new(net::mesh::mesh_task()));
    }

    // 7. Initialize UI (last, just before yielding control)
    #[cfg(feature = "gui")]
    gui::init_ui();

    serial_println!("Status: Multitasking active. System ready.");

    #[cfg(feature = "test")]
    run_tests();

    executor.run();
}

#[cfg(feature = "test")]
fn run_tests() {
    serial_println!("Running system tests...");
    test_println();
    test_pci_discovery();
    #[cfg(feature = "network")]
    jarvis_kernel::net::mesh::test_mesh_crypto();
    #[cfg(feature = "ai")]
    jarvis_kernel::ai::swarm::test_swarm_logic();

    jarvis_kernel::sensors::scene::test_scene_logic();

    #[cfg(feature = "storage")]
    jarvis_kernel::storage::dht::test_dht_storage();

    serial_println!("All tests passed!");
    jarvis_kernel::qemu::exit_qemu(jarvis_kernel::qemu::QemuExitCode::Success);
}

#[cfg(feature = "test")]
fn test_pci_discovery() {
    jarvis_kernel::serial_print!("test_pci_discovery... ");
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("PANIC: {}", info);
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
