use crate::apic::LocalApic;
use crate::gdt;
use crate::println;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spinning_top::Spinlock;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::VirtAddr;

pub const TIMER_INTERRUPT_VECTOR: u8 = 32;

pub static TICKS: AtomicU64 = AtomicU64::new(0);

/// Atomic storage for the APIC virtual base address to allow lock-free EOIs in ISRs.
pub static APIC_BASE: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    pub static ref LAPIC: Spinlock<Option<LocalApic>> = Spinlock::new(None);
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = TIMER_INTERRUPT_VECTOR,
    Keyboard,
    Spurious = 0xFF,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

/// Initializes the Local APIC.
///
/// # Safety
///
/// This function is unsafe because the caller must ensure that the
/// `physical_memory_offset` is correct and that the APIC base address
/// is mapped to the corresponding virtual address.
pub unsafe fn init_apic(physical_memory_offset: VirtAddr) {
    crate::apic::disable_pic();
    let mut lapic = LocalApic::new(physical_memory_offset);
    lapic.init();

    // Store the raw base address for lock-free EOI in ISRs
    // Standard Local APIC physical address is 0xFEE00000
    let base_addr = physical_memory_offset + 0xFEE0_0000u64;
    APIC_BASE.store(base_addr.as_u64(), Ordering::SeqCst);

    *LAPIC.lock() = Some(lapic);
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    // Exception handlers are sensitive. Avoid locks (println).
    // In a real system, we'd increment a lock-free diagnostic counter.
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // Standardizing output for stability watchdog (Wait, this is an exception, not a frequent ISR)
    // However, if we are in a deadlock state, println! will hang.
    // For critical failures (Page Fault, GPF), we'll keep the output but be aware of the risk.
    // Ideally, these would use a lock-free serial writer.
    println!("[CORE] EXCEPTION: GENERAL PROTECTION FAULT");
    println!("Error Code: 0x{:x}", error_code);
    println!("Instruction Pointer: {:?}", stack_frame.instruction_pointer);
    println!("{:#?}", stack_frame);
    panic!("GPF - System Halted for Safety");
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("[CORE] EXCEPTION: STACK SEGMENT FAULT");
    println!("Error Code: 0x{:x}", error_code);
    println!("{:#?}", stack_frame);
    panic!("SSF - System Halted for Safety");
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("[CORE] EXCEPTION: INVALID OPCODE");
    println!("At Address: {:?}", stack_frame.instruction_pointer);
    println!("{:#?}", stack_frame);
    panic!("UD - System Halted for Safety");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("[CORE] EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("Instruction Pointer: {:?}", stack_frame.instruction_pointer);
    println!("{:#?}", stack_frame);
    panic!("PAGE FAULT - System Halted for Safety");
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::SeqCst);

    // Lock-free EOI
    let base_addr = APIC_BASE.load(Ordering::SeqCst);
    if base_addr != 0 {
        unsafe {
            LocalApic::end_of_interrupt_raw(VirtAddr::new(base_addr));
        }
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    // Lock-free EOI
    let base_addr = APIC_BASE.load(Ordering::SeqCst);
    if base_addr != 0 {
        unsafe {
            LocalApic::end_of_interrupt_raw(VirtAddr::new(base_addr));
        }
    }
}
