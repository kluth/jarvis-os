use x86_64::{
    structures::paging::{PageTable, OffsetPageTable},
    VirtAddr, PhysAddr,
};

/// Initialisiert eine neue OffsetPageTable.
///
/// # Sicherheit
/// Diese Funktion ist unsicher, da der Aufrufer garantieren muss, dass der
/// gesamte physische Speicher unter dem übergebenen `physical_memory_offset`
/// gemappt ist. Außerdem darf diese Funktion nur einmal aufgerufen werden,
/// um mehrere mutable Aliase auf die Level-4-Tabelle zu verhindern.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Gibt eine mutable Referenz auf die aktive Level-4-Seitentabelle zurück.
///
/// # Sicherheit
/// Diese Funktion ist unsicher, da der Aufrufer garantieren muss, dass der
/// gesamte physische Speicher unter dem übergebenen `physical_memory_offset`
/// gemappt ist.
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

use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use bootloader_api::info::{MemoryRegions, MemoryRegionKind};

/// Ein FrameAllocator, der die Memory-Map des Bootloaders nutzt.
/// Er implementiert einen einfachen Bump-Allocator, der Frames sequenziell aus
/// den verfügbaren Speicherregionen zuweist.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Erstellt einen neuen FrameAllocator aus der übergebenen Memory-Map.
    ///
    /// # Sicherheit
    /// Diese Funktion ist unsicher, da der Aufrufer garantieren muss, dass die
    /// Memory-Map korrekt ist und dass alle als `Usable` markierten Frames
    /// tatsächlich unbenutzt sind.
    pub unsafe fn init(memory_map: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Gibt einen Iterator über die nutzbaren Frames in der Memory-Map zurück.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // Nutze die Memory-Map des Bootloaders
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.kind == MemoryRegionKind::Usable);
        // Transformiere die Regionen in einen Iterator über die Frame-Startadressen
        let addr_ranges = usable_regions
            .map(|r| r.start..r.end);
        // Transformiere die Adressbereiche in einen Iterator über 4KiB-Frames
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // Erzeuge `PhysFrame` Objekte aus den Adressen
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
