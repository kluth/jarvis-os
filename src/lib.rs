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
#[cfg(feature = "gui")]
pub mod gui_3d;
pub mod interrupts;
pub mod memory;
#[cfg(feature = "network")]
pub mod net;
#[cfg(feature = "gui")]
pub mod notifications;
pub mod pci;
pub mod qemu;
pub mod security;
pub mod sensors;
pub mod serial;
#[cfg(feature = "storage")]
pub mod storage;
pub mod task;
#[cfg(feature = "telemetry")]
pub mod telemetry;
#[cfg(feature = "gui")]
pub mod vga_buffer;

/// Enables SSE (Streaming SIMD Extensions) in the CPU.
///
/// # Safety
///
/// This function is unsafe because it directly modifies control registers (CR0, CR4).
/// It must be called during early boot to allow the compiler to safely use
/// 128-bit registers for math operations.
pub unsafe fn enable_sse() {
    use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

    // 1. Enable SSE by setting bits in CR4
    let mut cr4 = Cr4::read();
    cr4.insert(Cr4Flags::OSFXSR); // Support fxsave/fxrstor
    cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE); // Support unmasked SIMD exceptions
    Cr4::write(cr4);

    // 2. Ensure Coprocessor Emulation is disabled and Monitoring is enabled in CR0
    let mut cr0 = Cr0::read();
    cr0.remove(Cr0Flags::EMULATE_COPROCESSOR); // Clear EM bit
    cr0.insert(Cr0Flags::MONITOR_COPROCESSOR); // Set MP bit
    Cr0::write(cr0);
}

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
