use x86_64::PhysAddr;

/// A DMA-safe circular buffer for audio data.
/// It uses physically contiguous memory.
pub struct DmaBuffer {
    phys_addr: PhysAddr,
    size: usize,
    // We keep a pointer to the virtual address to write/read from the CPU
    virt_addr: *mut u8,
}

impl DmaBuffer {
    pub fn new(phys_addr: PhysAddr, virt_addr: *mut u8, size: usize) -> Self {
        DmaBuffer {
            phys_addr,
            virt_addr,
            size,
        }
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.phys_addr
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub unsafe fn write(&mut self, offset: usize, data: &[u8]) {
        let len = data.len();
        let target = self.virt_addr.add(offset % self.size);
        // Simple copy, doesn't handle wrap-around in one call yet
        core::ptr::copy_nonoverlapping(data.as_ptr(), target, len);
    }
}
