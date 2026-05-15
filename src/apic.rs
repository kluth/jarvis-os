use x86_64::VirtAddr;
use x86_64::instructions::port::Port;

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
    pub unsafe fn new(physical_memory_offset: VirtAddr) -> Self {
        // Standard Local APIC physical address is 0xFEE00000
        let base_addr = physical_memory_offset + 0xFEE0_0000u64;
        LocalApic { base_addr }
    }

    pub unsafe fn read(&self, reg: Register) -> u32 {
        let ptr = (self.base_addr + reg as u64).as_ptr() as *const u32;
        ptr.read_volatile()
    }

    pub unsafe fn write(&mut self, reg: Register, value: u32) {
        let ptr = (self.base_addr + reg as u64).as_mut_ptr() as *mut u32;
        ptr.write_volatile(value);
    }

    pub unsafe fn init(&mut self) {
        // Enable Local APIC by setting bit 8 in the Spurious Interrupt Vector Register
        let spurious_vector = 0xFF; // Common choice
        self.write(Register::SpuriousInterruptVector, self.read(Register::SpuriousInterruptVector) | 0x100 | spurious_vector);

        // Configure timer
        // Divide by 16
        self.write(Register::DivideConfiguration, 0x3);
        // Set LVT Timer register: periodic mode, vector 32 (Timer interrupt)
        self.write(Register::LvtTimer, 0x20000 | 32);
        // Set initial count (for now a fixed value, later calibrated)
        self.write(Register::InitialCount, 10_000_000);
    }

    pub unsafe fn end_of_interrupt(&mut self) {
        self.write(Register::EndOfInterrupt, 0);
    }
}

pub unsafe fn disable_pic() {
    let mut p1 = Port::<u8>::new(0xa1);
    let mut p2 = Port::<u8>::new(0x21);
    p1.write(0xff);
    p2.write(0xff);
}
