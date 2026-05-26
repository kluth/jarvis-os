#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::identity_op)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::double_parens)]
#![allow(clippy::unused_unit)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::redundant_guards)]
#![allow(clippy::assign_op_pattern)]
#![allow(unused_parens)]
#![allow(asm_sub_register)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_unsafe)]
#![allow(unused_attributes)]

extern crate alloc;

pub mod acpi;
pub mod ai;
pub mod allocator;
pub mod apic;
pub mod audio;
pub mod cros_ec;
pub mod device_manager;
pub mod entropy;
pub mod gdt;
pub mod gpu;
#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "gui")]
pub mod gui_3d;
pub mod i2c;
pub mod interrupts;
pub mod mem;
pub mod memory;
#[cfg(feature = "network")]
pub mod net;
#[cfg(feature = "gui")]
pub mod notifications;
pub mod pci;
pub mod qemu;
pub mod renderer;
pub mod security;
pub mod sensors;
pub mod serial;
#[cfg(feature = "storage")]
pub mod storage;
pub mod sync;
pub mod task;
#[cfg(feature = "telemetry")]
pub mod telemetry;
pub mod vga_buffer;
pub mod wasm;

pub use vga_buffer::WRITER;

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
    serial_println!("[STABILITY_CHECK:PANIC] Error: {}\n", info);
    qemu::exit_qemu(qemu::QemuExitCode::Failed);
}

/// Enable SSE (Streaming SIMD Extensions) support on x86_64.
///
/// # Safety
///
/// This function is unsafe because it directly modifies control registers (CR0 and CR4).
/// It must be called once during early boot.
pub unsafe fn enable_sse() {
    use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

    let mut cr4 = Cr4::read();
    cr4.insert(Cr4Flags::OSFXSR);
    cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
    Cr4::write(cr4);

    let mut cr0 = Cr0::read();
    cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
    cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
    Cr0::write(cr0);
}

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
