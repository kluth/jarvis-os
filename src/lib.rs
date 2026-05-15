#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod acpi;
#[cfg(feature = "ai")]
pub mod ai;
pub mod allocator;
pub mod apic;
#[cfg(feature = "audio")]
pub mod audio;
pub mod device_manager;
pub mod gdt;
#[cfg(feature = "gui")]
pub mod gui;
pub mod interrupts;
pub mod memory;
#[cfg(feature = "network")]
pub mod net;
pub mod pci;
pub mod qemu;
pub mod serial;
#[cfg(feature = "storage")]
pub mod storage;
pub mod task;
#[cfg(feature = "telemetry")]
pub mod telemetry;
#[cfg(feature = "gui")]
pub mod vga_buffer;

use core::panic::PanicInfo;

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    qemu::exit_qemu(qemu::QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    qemu::exit_qemu(qemu::QemuExitCode::Failed);
}

#[cfg(test)]
use bootloader_api::BootInfo;

#[cfg(test)]
bootloader_api::entry_point!(test_kernel_main);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
