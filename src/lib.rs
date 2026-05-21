#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod acpi;
pub mod ai;
pub mod allocator;
pub mod apic;
pub mod audio;
pub mod device_manager;
pub mod drivers;
pub mod gdt;
pub mod gui;
pub mod gui_3d;
pub mod hpet;
pub mod interrupts;
pub mod memory;
pub mod net;
pub mod notifications;
pub mod pci;
pub mod qemu;
pub mod security;
pub mod sensors;
pub mod storage;
pub mod sync;
pub mod task;
pub mod telemetry;
pub mod vga_buffer;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

static HEAP_READY: AtomicBool = AtomicBool::new(false);

pub fn set_heap_ready() {
    HEAP_READY.store(true, Ordering::SeqCst);
}
pub fn is_heap_ready() -> bool {
    HEAP_READY.load(Ordering::SeqCst)
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        crate::serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        crate::serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    crate::serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    crate::qemu::exit_qemu(crate::qemu::QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    crate::serial_println!("[failed]\n");
    crate::serial_println!("Error: {}\n", info);
    crate::qemu::exit_qemu(crate::qemu::QemuExitCode::Failed);
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn enable_sse() {
    use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

    unsafe {
        let mut cr0 = Cr0::read();
        cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
        cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
        Cr0::write(cr0);

        let mut cr4 = Cr4::read();
        cr4.insert(Cr4Flags::OSFXSR);
        cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
        Cr4::write(cr4);
    }
}
