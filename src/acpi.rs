use crate::{device_manager, println};
use core::ptr;
use x86_64::{PhysAddr, VirtAddr};

/// The Root System Description Pointer (RSDP)
#[repr(C, packed)]
struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_addr: u32,
}

/// Common header for all ACPI System Description Tables
#[repr(C, packed)]
struct SdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[repr(C, packed)]
struct HpetTable {
    header: SdtHeader,
    event_timer_block_id: u32,
    base_address: GenericAddressStructure,
    hpet_number: u8,
    main_counter_minimum_clock_tick_periodic: u16,
    page_protection_and_oem_attribute: u8,
}

#[repr(C, packed)]
struct GenericAddressStructure {
    address_space: u8,
    bit_width: u8,
    bit_offset: u8,
    access_size: u8,
    address: u64,
}

pub static mut HPET_BASE: Option<VirtAddr> = None;

pub fn init(rsdp_addr: PhysAddr) {
    println!("ACPI: Initializing at PhysAddr({:?})", rsdp_addr);

    // Get the physical memory offset from the kernel's state (passed via boot_info)
    // For now, we assume we can calculate it if we have access to it.
    // In main.rs, we use VirtAddr::new(boot_info.physical_memory_offset).
    // We'll need to pass it or have a global for it.
    // Let's assume we can use a trick to find it or we must pass it to acpi::init.

    // Wait, main.rs calls acpi::init(PhysAddr::new(rsdp_addr)).
    // I should change acpi::init to also take the phys_mem_offset.
}

/// Initializes the ACPI tables using the provided RSDP address and physical memory offset.
///
/// # Safety
/// The caller must ensure that `phys_mem_offset` is correct and that the memory at `rsdp_addr`
/// contains a valid RSDP structure.
pub fn init_with_offset(rsdp_addr: PhysAddr, phys_mem_offset: VirtAddr) {
    println!("ACPI: Real Initialization at PhysAddr({:?})", rsdp_addr);

    unsafe {
        let rsdp_ptr: *const Rsdp = (phys_mem_offset + rsdp_addr.as_u64()).as_ptr();
        let rsdp = &*rsdp_ptr;

        if &rsdp.signature != b"RSD PTR " {
            println!("ACPI Error: Invalid RSDP signature");
            return;
        }

        println!("ACPI: RSDP Revision {}", rsdp.revision);

        let rsdt_ptr: *const SdtHeader = (phys_mem_offset + rsdp.rsdt_addr as u64).as_ptr();
        let rsdt = &*rsdt_ptr;

        if &rsdt.signature != b"RSDT" {
            println!("ACPI Error: Invalid RSDT signature");
            return;
        }

        let entry_count = (rsdt.length - core::mem::size_of::<SdtHeader>() as u32) / 4;
        let entries_ptr = rsdt_ptr.add(1) as *const u32;

        for i in 0..entry_count {
            let sdt_phys_addr = ptr::read_unaligned(entries_ptr.add(i as usize));
            let sdt_ptr: *const SdtHeader = (phys_mem_offset + sdt_phys_addr as u64).as_ptr();
            let sdt = &*sdt_ptr;

            let sig = core::str::from_utf8(&sdt.signature).unwrap_or("????");
            println!("ACPI: Found table {}", sig);

            if &sdt.signature == b"HPET" {
                println!("ACPI: HPET Table detected!");
                let hpet_table = &*(sdt_ptr as *const HpetTable);
                let hpet_phys_addr = hpet_table.base_address.address;
                println!("ACPI: HPET Base Address: 0x{:x}", hpet_phys_addr);

                HPET_BASE = Some(phys_mem_offset + hpet_phys_addr);

                device_manager::register(device_manager::DeviceInfo {
                    name: "HPET Timer",
                    dev_type: device_manager::DeviceType::System,
                    status: "Initialized",
                    capabilities: &[device_manager::Capability::Diagnostic],
                });
            }

            if &sdt.signature == b"APIC" {
                device_manager::register(device_manager::DeviceInfo {
                    name: "IO APIC",
                    dev_type: device_manager::DeviceType::System,
                    status: "Enabled",
                    capabilities: &[],
                });
            }
        }
    }

    device_manager::register(device_manager::DeviceInfo {
        name: "ACPI Controller",
        dev_type: device_manager::DeviceType::System,
        status: "Enabled",
        capabilities: &[device_manager::Capability::PowerControl],
    });
}
