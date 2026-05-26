use x86_64::instructions::port::Port;
use x86_64::VirtAddr;

/// Standard Local APIC physical address is 0xFEE00000
pub const DEFAULT_APIC_PHYS_BASE: u64 = 0xFEE0_0000;

/// Local APIC registers offsets
#[repr(usize)]
pub enum Register {
    Id = 0x20,
    Version = 0x30,
    TaskPriority = 0x80,
    ArbitrationPriority = 0x90,
    ProcessorPriority = 0xA0,
    EndOfInterrupt = 0xB0,
    RemoteRead = 0xC0,
    LogicalDestination = 0xD0,
    DestinationFormat = 0xE0,
    SpuriousInterruptVector = 0xF0,
    InService = 0x100,
    TriggerMode = 0x180,
    InterruptRequest = 0x200,
    ErrorStatus = 0x280,
    LvtCorrectedMachineCheckInterrupt = 0x2F0,
    InterruptCommand = 0x300,
    LvtTimer = 0x320,
    LvtThermalSensor = 0x330,
    LvtPerformanceMonitoringCounters = 0x340,
    LvtLint0 = 0x350,
    LvtLint1 = 0x360,
    LvtError = 0x370,
    InitialCount = 0x380,
    CurrentCount = 0x390,
    DivideConfiguration = 0x3E0,
}

pub struct LocalApic {
    base_addr: VirtAddr,
}

impl LocalApic {
    /// Creates a new LocalApic instance.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `physical_memory_offset` is correct and that
    /// the Local APIC memory region is mapped at the resulting virtual address.
    pub unsafe fn new(physical_memory_offset: VirtAddr) -> Self {
        let base_addr = physical_memory_offset + DEFAULT_APIC_PHYS_BASE;
        LocalApic { base_addr }
    }

    /// Reads a value from a Local APIC register.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the register is valid and that the APIC
    /// base address is correctly initialized.
    pub unsafe fn read(&self, reg: Register) -> u32 {
        let ptr: *const u32 = (self.base_addr + reg as u64).as_ptr();
        ptr.read_volatile()
    }

    /// Writes a value to a Local APIC register.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the register is valid and that the APIC
    /// base address is correctly initialized.
    pub unsafe fn write(&mut self, reg: Register, value: u32) {
        let ptr: *mut u32 = (self.base_addr + reg as u64).as_mut_ptr();
        ptr.write_volatile(value);
    }

    /// Initializes the Local APIC.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the APIC is correctly mapped and accessible.
    pub unsafe fn init(&mut self) {
        // Enable Local APIC by setting bit 8 in the Spurious Interrupt Vector Register
        let spurious_vector = 0xFF; // Common choice
        self.write(
            Register::SpuriousInterruptVector,
            self.read(Register::SpuriousInterruptVector) | 0x100 | spurious_vector,
        );

        // Configure timer
        // Divide by 16
        self.write(Register::DivideConfiguration, 0x3);
        // Set LVT Timer register: periodic mode, vector 32 (Timer interrupt)
        self.write(Register::LvtTimer, 0x20000 | 32);
        // Set initial count (for now a fixed value, later calibrated)
        self.write(Register::InitialCount, 10_000_000);
    }

    /// Signals End of Interrupt (EOI) to the Local APIC.
    ///
    /// # Safety
    ///
    /// This must be called at the end of every hardware interrupt handled by the APIC.
    pub unsafe fn end_of_interrupt(&mut self) {
        Self::end_of_interrupt_raw(self.base_addr);
    }

    /// Signals End of Interrupt (EOI) to the Local APIC using a raw virtual address.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `base_addr` is the correct virtual address of the APIC.
    pub unsafe fn end_of_interrupt_raw(base_addr: VirtAddr) {
        let ptr: *mut u32 = (base_addr + Register::EndOfInterrupt as u64).as_mut_ptr();
        ptr.write_volatile(0);
    }
}

/// Disables the legacy 8259 PIC.
///
/// # Safety
///
/// This involves raw I/O port access and should only be called during early boot
/// when switching to the APIC.
pub unsafe fn disable_pic() {
    let mut p1 = Port::<u8>::new(0xa1);
    let mut p2 = Port::<u8>::new(0x21);
    p1.write(0xff);
    p2.write(0xff);
}
