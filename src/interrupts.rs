use crate::apic::LocalApic;
use crate::gdt;
use crate::println;
use lazy_static::lazy_static;
use spinning_top::Spinlock;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::VirtAddr;

pub const TIMER_INTERRUPT_VECTOR: u8 = 32;

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
    *LAPIC.lock() = Some(lapic);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    panic!("PAGE FAULT");
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    if let Some(ref mut lapic) = *LAPIC.lock() {
        unsafe {
            lapic.end_of_interrupt();
        }
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    if let Some(ref mut lapic) = *LAPIC.lock() {
        unsafe {
            lapic.end_of_interrupt();
        }
    }
}
