use crate::acpi::HPET_BASE;
use x86_64::VirtAddr;

pub struct Hpet {
    base_addr: VirtAddr,
}

impl Hpet {
    /// Creates a new HPET instance.
    ///
    /// # Safety
    /// The caller must ensure the base address is a valid mapping to the HPET MMIO region.
    pub unsafe fn new(base_addr: VirtAddr) -> Self {
        Self { base_addr }
    }

    /// Initializes the HPET.
    ///
    /// # Safety
    /// This function must be called only after the HPET base address has been correctly mapped.
    pub unsafe fn init(&self) {
        // 1. Enable the main counter
        let config = self.read_reg(0x10);
        self.write_reg(0x10, config | 1);
    }

    pub fn read_main_counter(&self) -> u64 {
        unsafe { self.read_reg(0xF0) }
    }

    pub fn get_period_fs(&self) -> u32 {
        unsafe { (self.read_reg(0x00) >> 32) as u32 }
    }

    unsafe fn read_reg(&self, offset: u64) -> u64 {
        let ptr = (self.base_addr + offset).as_ptr::<u64>();
        ptr.read_volatile()
    }

    unsafe fn write_reg(&self, offset: u64, val: u64) {
        let ptr = (self.base_addr + offset).as_mut_ptr::<u64>();
        ptr.write_volatile(val);
    }
}

pub fn get_nanoseconds() -> u64 {
    unsafe {
        if let Some(base) = HPET_BASE {
            let hpet = Hpet::new(base);
            let counter = hpet.read_main_counter();
            let period = hpet.get_period_fs();
            // counter * period / 1,000,000 (to get nanoseconds from femtoseconds)
            (counter as u128 * period as u128 / 1_000_000) as u64
        } else {
            0
        }
    }
}
