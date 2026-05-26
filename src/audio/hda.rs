use crate::device_manager::Device;
use crate::println;
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
    pub unsafe fn new(dev: &Device, physical_memory_offset: VirtAddr) -> Self {
        let bar0 = dev.bars[0].base;
        let base_addr = physical_memory_offset + bar0;

        // Enable Bus Mastering
        let mut cmd = crate::pci::pci_read_word(dev.bus, dev.slot, dev.function, 0x04);
        cmd |= 0x07; // I/O Space | Memory Space | Bus Master
        crate::pci::pci_write_word(dev.bus, dev.slot, dev.function, 0x04, cmd);

        HdaController { base_addr }
    }

    /// Initializes the HDA controller.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the controller base address is valid and mapped.
    pub unsafe fn init(&mut self) {
        println!("Initializing HDA Controller at {:?}", self.base_addr);

        // 1. Reset the controller
        let gctl_ptr: *mut u32 = self.base_addr.as_mut_ptr();

        // Clear CRST bit (bit 0) to reset
        unsafe {
            gctl_ptr.write_volatile(gctl_ptr.read_volatile() & !1);
            let mut timeout = 100_000;
            while gctl_ptr.read_volatile() & 1 != 0 && timeout > 0 {
                timeout -= 1;
                core::hint::spin_loop();
            }
            if timeout == 0 {
                println!("HDA Warning: Reset timeout (entering reset)");
            }

            // Set CRST bit to 1 to exit reset
            gctl_ptr.write_volatile(gctl_ptr.read_volatile() | 1);
            timeout = 100_000;
            while gctl_ptr.read_volatile() & 1 == 0 && timeout > 0 {
                timeout -= 1;
                core::hint::spin_loop();
            }
            if timeout == 0 {
                println!("HDA Warning: Reset timeout (exiting reset)");
            }
        }

        println!("HDA Controller exited reset state.");

        // 2. Initialize CORB/RIRB (Command/Response Ring Buffers)
        // For now, we just print the capabilities. Full DMA allocation requires
        // a more complex memory setup which will be done in the next phase.
        let corbsize: *const u8 = (self.base_addr + 0x4Eu64).as_ptr();
        println!("CORB Size Capability: 0x{:x}", corbsize.read_volatile());

        let rirbsize: *const u8 = (self.base_addr + 0x5Eu64).as_ptr();
        println!("RIRB Size Capability: 0x{:x}", rirbsize.read_volatile());
    }
}
