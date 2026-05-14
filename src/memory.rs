use x86_64::{
    structures::paging::{PageTable, OffsetPageTable, FrameAllocator, PhysFrame, Size4KiB, FrameDeallocator},
    VirtAddr, PhysAddr,
};
use bootloader_api::info::{MemoryRegions, MemoryRegionKind};

/// Initializes a new OffsetPageTable.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to the specified `physical_memory_offset`.
/// Also, this function must be only called once to avoid aliasing mutable
/// references to the level 4 table.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 page table.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to the specified `physical_memory_offset`.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub struct BitmapFrameAllocator {
    bitmap: &'static mut [u8],
    max_frame_index: usize,
    next_search_index: usize,
}

impl BitmapFrameAllocator {
    /// Create a BitmapFrameAllocator from the passed memory map.
    ///
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid and that the physical memory offset is correct.
    pub unsafe fn init(memory_map: &'static MemoryRegions, physical_memory_offset: VirtAddr) -> Self {
        let mut max_addr = 0;
        for region in memory_map.iter() {
            if region.end > max_addr {
                max_addr = region.end;
            }
        }

        let total_frames = (max_addr / 4096) as usize;
        let bitmap_size = (total_frames + 7) / 8;

        // Find a usable region large enough for the bitmap
        let mut bitmap_addr = 0;
        for region in memory_map.iter() {
            if region.kind == MemoryRegionKind::Usable && (region.end - region.start) as usize >= bitmap_size {
                bitmap_addr = region.start;
                break;
            }
        }

        if bitmap_addr == 0 {
            panic!("Could not find a usable memory region for the frame allocator bitmap");
        }

        let bitmap_ptr = (physical_memory_offset + bitmap_addr).as_mut_ptr() as *mut u8;
        let bitmap = core::slice::from_raw_parts_mut(bitmap_ptr, bitmap_size);

        // Initialize bitmap: all frames used
        bitmap.fill(0xFF);

        let mut allocator = BitmapFrameAllocator {
            bitmap,
            max_frame_index: total_frames,
            next_search_index: 0,
        };

        // Mark usable regions as free in the bitmap
        for region in memory_map.iter() {
            if region.kind == MemoryRegionKind::Usable {
                let start_frame = (region.start / 4096) as usize;
                let end_frame = (region.end / 4096) as usize;
                for i in start_frame..end_frame {
                    allocator.set_free(i);
                }
            }
        }

        // Mark the bitmap itself as used
        let bitmap_start_frame = (bitmap_addr / 4096) as usize;
        let bitmap_end_frame = ((bitmap_addr + bitmap_size as u64 + 4095) / 4096) as usize;
        for i in bitmap_start_frame..bitmap_end_frame {
            allocator.set_used(i);
        }

        // Frame 0 is usually special/reserved, mark it used just in case
        allocator.set_used(0);

        allocator
    }

    fn set_free(&mut self, frame_index: usize) {
        if frame_index < self.max_frame_index {
            self.bitmap[frame_index / 8] &= !(1 << (frame_index % 8));
        }
    }

    fn set_used(&mut self, frame_index: usize) {
        if frame_index < self.max_frame_index {
            self.bitmap[frame_index / 8] |= 1 << (frame_index % 8);
        }
    }

    fn is_used(&self, frame_index: usize) -> bool {
        if frame_index >= self.max_frame_index {
            return true;
        }
        (self.bitmap[frame_index / 8] & (1 << (frame_index % 8))) != 0
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Linear scan starting from next_search_index
        for i in 0..self.max_frame_index {
            let index = (self.next_search_index + i) % self.max_frame_index;
            if !self.is_used(index) {
                self.set_used(index);
                self.next_search_index = (index + 1) % self.max_frame_index;
                return Some(PhysFrame::containing_address(PhysAddr::new((index as u64) * 4096)));
            }
        }
        None
    }
}

impl FrameDeallocator<Size4KiB> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let index = (frame.start_address().as_u64() / 4096) as usize;
        self.set_free(index);
    }
}
