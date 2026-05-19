use crate::pci::PciDevice;
use x86_64::VirtAddr;

pub struct HdaController {
    base_addr: VirtAddr,
}

impl HdaController {
    /// Creates a new HdaController instance from a PCI device.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `dev` is a valid HDA controller and that
    /// `physical_memory_offset` is correct.
    pub unsafe fn new(dev: &PciDevice, physical_memory_offset: VirtAddr) -> Self {
        let bar0 = dev.read_bar(0) & 0xFFFF_FFF0; // Mask out flags
        let base_addr = physical_memory_offset + bar0 as u64;

        dev.enable_bus_mastering();

        HdaController { base_addr }
    }

    /// Initializes the HDA controller.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the controller base address is valid and mapped.
    pub unsafe fn init(&mut self) {
        crate::serial_println!("Initializing HDA Controller at {:?}", self.base_addr);

        // 1. Reset the controller
        let gctl_ptr: *mut u32 = self.base_addr.as_mut_ptr();

        // Clear CRST bit (bit 0) to reset
        gctl_ptr.write_volatile(gctl_ptr.read_volatile() & !1);
        while gctl_ptr.read_volatile() & 1 != 0 {}

        // Set CRST bit to 1 to exit reset
        gctl_ptr.write_volatile(gctl_ptr.read_volatile() | 1);
        while gctl_ptr.read_volatile() & 1 == 0 {}

        crate::serial_println!("HDA Controller exited reset state.");

        // 2. Initialize CORB/RIRB (Command/Response Ring Buffers)
        // For now, we just print the capabilities. Full DMA allocation requires
        // a more complex memory setup which will be done in the next phase.
        let corbsize: *const u8 = (self.base_addr + 0x4Eu64).as_ptr();
        crate::serial_println!("CORB Size Capability: 0x{:x}", corbsize.read_volatile());

        let rirbsize: *const u8 = (self.base_addr + 0x5Eu64).as_ptr();
        crate::serial_println!("RIRB Size Capability: 0x{:x}", rirbsize.read_volatile());
    }
}
