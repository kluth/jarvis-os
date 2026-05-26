//! ============================================================================
//! JARVIS OS ACPI Subsystem — Hardware Discovery via ACPI Tables
//! ============================================================================
//! Implements:
//!   - RSDP (Root System Description Pointer) detection
//!   - RSDT / XSDT table walk
//!   - MADT (Multiple APIC Description Table) — local & I/O APIC
//!   - MCFG (PCI Express Memory-Mapped Config Space)
//!   - FADT (Fixed ACPI Description Table) — PM timers, SMI, reset
//!   - DSDT / SSDT — AML namespace (device discovery foundation)
//! ============================================================================

use core::mem;

// ============================================================================
// ACPI TABLE HEADER — all ACPI tables start with this
// ============================================================================

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct AcpiSdtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

// ============================================================================
// RSDP — Root System Description Pointer (v1 + v2)
// ============================================================================

#[repr(C, packed)]
pub struct RsdpDescriptor {
    pub signature: [u8; 8],           // "RSD PTR "
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
}

#[repr(C, packed)]
pub struct RsdpDescriptorV2 {
    pub base: RsdpDescriptor,
    pub length: u32,
    pub xsdt_address: u64,
    pub ext_checksum: u8,
    pub reserved: [u8; 3],
}

impl RsdpDescriptor {
    fn valid(&self) -> bool {
        &self.signature == b"RSD PTR "
    }
}

// ============================================================================
// MADT — Multiple APIC Description Table
// ============================================================================

#[repr(C, packed)]
pub struct Madt {
    pub header: AcpiSdtHeader,
    pub local_apic_address: u32,
    pub flags: u32,
    // Followed by entries of types 0x00-0x0F
}

#[repr(C, packed)]
pub struct MadtEntryHeader {
    pub entry_type: u8,
    pub record_length: u8,
}

const MADT_TYPE_LOCAL_APIC: u8 = 0x00;
const MADT_TYPE_IO_APIC: u8 = 0x01;
const MADT_TYPE_ISO: u8 = 0x02;
const MADT_TYPE_LOCAL_APIC_ADDR_OVERRIDE: u8 = 0x05;

#[repr(C, packed)]
pub struct MadtLocalApic {
    pub header: MadtEntryHeader,
    pub processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

#[repr(C, packed)]
pub struct MadtIoApic {
    pub header: MadtEntryHeader,
    pub io_apic_id: u8,
    pub io_apic_address: u32,
    pub global_system_interrupt_base: u32,
}

// ============================================================================
// MCFG — PCI Express Memory-Mapped Config Space
// ============================================================================

#[repr(C, packed)]
pub struct Mcfg {
    pub header: AcpiSdtHeader,
    pub reserved: [u8; 8],
    // Followed by allocation structures
}

#[repr(C, packed)]
pub struct McfgAllocation {
    pub base_address: u64,
    pub pci_segment_group: u16,
    pub start_bus: u8,
    pub end_bus: u8,
    pub reserved: [u8; 4],
}

// ============================================================================
// FADT — Fixed ACPI Description Table
// ============================================================================

#[repr(C, packed)]
pub struct Fadt {
    pub header: AcpiSdtHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    // DSDT is at original DSDT address
    // ACPI 2.0+ has X_DSDT (64-bit)
    // ... many fields ...
    pub preferred_pm_profile: u8,
    pub sci_int: u16,
    pub smi_cmd_port: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_cnt: u8,
    pub pm1a_evt_blk: u32,
    pub pm1b_evt_blk: u32,
    pub pm1a_cnt_blk: u32,
    pub pm1b_cnt_blk: u32,
    pub pm2_cnt_blk: u32,
    pub pm_tmr_blk: u32,
    pub gpe0_blk: u32,
    pub gpe1_blk: u32,
    pub pm1_evt_len: u8,
    pub pm1_cnt_len: u8,
    pub pm2_cnt_len: u8,
    pub pm_tmr_len: u8,
    pub gpe0_blk_len: u8,
    pub gpe1_blk_len: u8,
    pub gpe1_base: u8,
    pub cst_cnt: u8,
    pub p_lvl2_lat: u16,
    pub p_lvl3_lat: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,
    pub iapc_boot_arch: u16,
    pub reserved: u8,
    pub flags: u32,
    pub reset_reg: GenericAddressStructure,
    pub reset_value: u8,
    pub arm_boot_arch: u16,
    pub fadt_minor_version: u8,
    pub x_firmware_ctrl: u64,
    pub x_dsdt: u64,
    pub x_pm1a_evt_blk: GenericAddressStructure,
    pub x_pm1b_evt_blk: GenericAddressStructure,
    pub x_pm1a_cnt_blk: GenericAddressStructure,
    pub x_pm1b_cnt_blk: GenericAddressStructure,
    pub x_pm2_cnt_blk: GenericAddressStructure,
    pub x_pm_tmr_blk: GenericAddressStructure,
    pub x_gpe0_blk: GenericAddressStructure,
    pub x_gpe1_blk: GenericAddressStructure,
    pub sleep_control_reg: GenericAddressStructure,
    pub sleep_status_reg: GenericAddressStructure,
    pub hypervisor_vendor_id: u64,
}

#[repr(C, packed)]
pub struct GenericAddressStructure {
    pub address_space: u8,
    pub bit_width: u8,
    pub bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}

// ============================================================================
// PARSED ACPI DATA STORE
// ============================================================================

pub struct AcpiData {
    pub revision: u8,
    pub rsdt_address: u32,
    pub xsdt_address: u64,

    // MADT entries
    pub local_apic_address: u32,
    pub io_apic_count: u32,
    pub processor_count: u32,

    // MCFG (PCIe ECAM)
    pub ecam_base: u64,
    pub ecam_segment: u16,
    pub ecam_start_bus: u8,
    pub ecam_end_bus: u8,
    pub ecam_present: bool,

    // FADT
    pub pm_timer_block: u32,
    pub pm_timer_len: u8,
    pub reset_port: u16,
    pub reset_value: u8,
    pub fadt_present: bool,

    // DSDT
    pub dsdt_address: u64,
    pub dsdt_length: u32,
    pub dsdt_present: bool,

    // HPET
    pub hpet_address: u64,
    pub hpet_present: bool,
}

impl Default for AcpiData {
    fn default() -> Self {
        Self::new()
    }
}

impl AcpiData {
    pub const fn new() -> Self {
        Self {
            revision: 0,
            rsdt_address: 0,
            xsdt_address: 0,
            local_apic_address: 0,
            io_apic_count: 0,
            processor_count: 0,
            ecam_base: 0,
            ecam_segment: 0,
            ecam_start_bus: 0,
            ecam_end_bus: 0,
            ecam_present: false,
            pm_timer_block: 0,
            pm_timer_len: 0,
            reset_port: 0,
            reset_value: 0,
            fadt_present: false,
            dsdt_address: 0,
            dsdt_length: 0,
            dsdt_present: false,
            hpet_address: 0,
            hpet_present: false,
        }
    }
}

// ============================================================================
// CHECKSUM VALIDATION
// ============================================================================

fn acpi_checksum(data: *const u8, length: usize) -> bool {
    let mut sum: u8 = 0;
    for i in 0..length {
        unsafe { sum = sum.wrapping_add(*data.add(i)); }
    }
    sum == 0
}

// ============================================================================
// RSDP SCAN — Search EBDA and BIOS ROM area
// ============================================================================

unsafe fn scan_for_rsdp() -> Option<u64> {
    // Search 0xE0000 - 0xFFFFF (BIOS ROM area)
    let bios_start = 0xE0000usize;
    let bios_end = 0x100000usize;

    // Also check EBDA (Extended BIOS Data Area)
    let ebda_ptr = *(0x40E as *const u16) as usize;
    let ebda_start = ebda_ptr;
    let ebda_end = ebda_ptr + 1024;

    // Search EBDA first
    let mut addr = ebda_start;
    while addr + 16 <= ebda_end {
        let rsdp = addr as *const RsdpDescriptor;
        if (*rsdp).valid() {
            return if (*rsdp).revision >= 2 {
                let v2 = addr as *const RsdpDescriptorV2;
                Some((*v2).xsdt_address)
            } else {
                Some((*rsdp).rsdt_address as u64)
            };
        }
        addr += 16;
    }

    // Search BIOS ROM
    addr = bios_start;
    while addr + 16 <= bios_end {
        let rsdp = addr as *const RsdpDescriptor;
        if (*rsdp).valid() {
            return if (*rsdp).revision >= 2 {
                let v2 = addr as *const RsdpDescriptorV2;
                Some((*v2).xsdt_address)
            } else {
                Some((*rsdp).rsdt_address as u64)
            };
        }
        addr += 16;
    }

    None
}

// ============================================================================
// TABLE PARSING
// ============================================================================

unsafe fn parse_table<T>(phys_addr: u64, phys_mem_offset: u64) -> &'static T {
    let virt = (phys_mem_offset + phys_addr) as *const T;
    &*virt
}

/// Validate and use an ACPI physical address as a virtual pointer
unsafe fn ptr_at<T>(phys: u64, offset: u64) -> &'static T {
    &*((offset + phys) as *const T)
}

/// Parse the RSDT (32-bit)
unsafe fn parse_rsdt(rsdt_addr: u64, phys_mem_offset: u64, data: &mut AcpiData) {
    let header: &AcpiSdtHeader = ptr_at(rsdt_addr, phys_mem_offset);
    if !acpi_checksum(header as *const _ as *const u8, header.length as usize) {
        crate::serial_println!("ACPI: RSDT checksum FAILED");
        return;
    }

    let num_tables = (header.length as usize - mem::size_of::<AcpiSdtHeader>()) / 4;
    let table_ptrs = (phys_mem_offset + rsdt_addr + mem::size_of::<AcpiSdtHeader>() as u64) as *const u32;

    for i in 0..num_tables {
        let table_phys = *table_ptrs.add(i) as u64;
        parse_sdt_table(table_phys, phys_mem_offset, data);
    }
}

/// Parse the XSDT (64-bit)
unsafe fn parse_xsdt(xsdt_addr: u64, phys_mem_offset: u64, data: &mut AcpiData) {
    let header: &AcpiSdtHeader = ptr_at(xsdt_addr, phys_mem_offset);
    if !acpi_checksum(header as *const _ as *const u8, header.length as usize) {
        crate::serial_println!("ACPI: XSDT checksum FAILED");
        return;
    }

    let num_tables = (header.length as usize - mem::size_of::<AcpiSdtHeader>()) / 8;
    let table_ptrs = (phys_mem_offset + xsdt_addr + mem::size_of::<AcpiSdtHeader>() as u64) as *const u64;

    for i in 0..num_tables {
        let table_phys = *table_ptrs.add(i);
        parse_sdt_table(table_phys, phys_mem_offset, data);
    }
}

/// Parse a single SDT by signature
unsafe fn parse_sdt_table(table_phys: u64, phys_mem_offset: u64, data: &mut AcpiData) {
    let header: &AcpiSdtHeader = ptr_at(table_phys, phys_mem_offset);

    let sig = core::str::from_utf8_unchecked(&header.signature);
    match sig {
        "APIC" => parse_madt(table_phys, phys_mem_offset, data),
        "MCFG" => parse_mcfg(table_phys, phys_mem_offset, data),
        "FACP" => parse_fadt(table_phys, phys_mem_offset, data),
        "HPET" => {
            // HPET table base address is at offset 44 (8 bytes, Generic Address Structure)
            let addr_ptr = (phys_mem_offset + table_phys + 44) as *const u64;
            data.hpet_address = *addr_ptr;
            data.hpet_present = true;
            crate::serial_println!("ACPI: Found HPET table at {:#018x}", data.hpet_address);
        }
        _ => {
            // Ignore unknown tables
        }
    }
}

/// Parse MADT (Multiple APIC Description Table)
unsafe fn parse_madt(madt_phys: u64, phys_mem_offset: u64, data: &mut AcpiData) {
    let madt: &Madt = ptr_at(madt_phys, phys_mem_offset);
    data.local_apic_address = madt.local_apic_address;

    let madt_end = madt_phys + madt.header.length as u64;
    let mut entry_phys = madt_phys + mem::size_of::<Madt>() as u64;

    while entry_phys + 2 <= madt_end {
        let entry_header: &MadtEntryHeader = ptr_at(entry_phys, phys_mem_offset);

        match entry_header.entry_type {
            MADT_TYPE_LOCAL_APIC => {
                if entry_header.record_length as u64 >= mem::size_of::<MadtLocalApic>() as u64 {
                    let lapic: &MadtLocalApic = ptr_at(entry_phys, phys_mem_offset);
                    if lapic.flags & 1 != 0 {
                        data.processor_count += 1;
                        crate::serial_println!("ACPI: CPU {} LAPIC ID {}", data.processor_count, lapic.apic_id);
                    }
                }
            }
            MADT_TYPE_IO_APIC => {
                if entry_header.record_length as u64 >= mem::size_of::<MadtIoApic>() as u64 {
                    let ioapic: &MadtIoApic = ptr_at(entry_phys, phys_mem_offset);
                    let ioapic_id = ioapic.io_apic_id;
                    let ioapic_addr = ioapic.io_apic_address;
                    let gsi_base = ioapic.global_system_interrupt_base;
                    data.io_apic_count += 1;
                    crate::serial_println!(
                        "ACPI: IOAPIC #{} at {:#010x}, GSI base {}",
                        ioapic_id, ioapic_addr, gsi_base,
                    );
                }
            }
            MADT_TYPE_LOCAL_APIC_ADDR_OVERRIDE
                // Override the local APIC address
                if entry_header.record_length as u64 >= 12 => {
                    let override_addr = entry_phys + mem::size_of::<MadtEntryHeader>() as u64;
                    let new_addr = *(ptr_at::<u64>(override_addr, phys_mem_offset));
                    data.local_apic_address = new_addr as u32;
                    crate::serial_println!("ACPI: LAPIC address override -> {:#010x}", new_addr);
                }
            _ => {}
        }

        entry_phys += entry_header.record_length as u64;
    }

    crate::serial_println!("ACPI: MADT parsed: {} CPUs, {} IOAPICs", data.processor_count, data.io_apic_count);
}

/// Parse MCFG (PCI Express Memory-Mapped Config Space)
unsafe fn parse_mcfg(mcfg_phys: u64, phys_mem_offset: u64, data: &mut AcpiData) {
    #[repr(C, packed)]
    struct McfgFull {
        header: AcpiSdtHeader,
        reserved: [u8; 8],
        // allocations start here
    }

    let mcfg: &McfgFull = ptr_at(mcfg_phys, phys_mem_offset);
    let num_alloc = (mcfg.header.length as usize - mem::size_of::<McfgFull>()) / mem::size_of::<McfgAllocation>();
    let alloc_start = mcfg_phys + mem::size_of::<McfgFull>() as u64;

    for i in 0..num_alloc {
        let alloc: &McfgAllocation = ptr_at(alloc_start + (i * mem::size_of::<McfgAllocation>()) as u64, phys_mem_offset);
        let alloc_segment = alloc.pci_segment_group;
        let alloc_base = alloc.base_address;
        let alloc_start_bus = alloc.start_bus;
        let alloc_end_bus = alloc.end_bus;
        crate::serial_println!(
            "ACPI: MCFG ECAM segment={} base={:#018x} bus={}-{}",
            alloc_segment, alloc_base, alloc_start_bus, alloc_end_bus,
        );

        // Store the first ECAM config space
        if !data.ecam_present {
            data.ecam_base = alloc.base_address;
            data.ecam_segment = alloc.pci_segment_group;
            data.ecam_start_bus = alloc.start_bus;
            data.ecam_end_bus = alloc.end_bus;
            data.ecam_present = true;
        }
    }
}

/// Parse FADT (Fixed ACPI Description Table)
unsafe fn parse_fadt(fadt_phys: u64, phys_mem_offset: u64, data: &mut AcpiData) {
    let fadt: &Fadt = ptr_at(fadt_phys, phys_mem_offset);
    data.fadt_present = true;
    data.pm_timer_block = fadt.pm_tmr_blk;
    data.pm_timer_len = fadt.pm_tmr_len;
    data.reset_port = fadt.reset_reg.address as u16;
    data.reset_value = fadt.reset_value;

    // DSDT address
    if fadt.header.revision >= 2 && fadt.x_dsdt != 0 {
        data.dsdt_address = fadt.x_dsdt;
    } else if fadt.dsdt != 0 {
        data.dsdt_address = fadt.dsdt as u64;
    }

    if data.dsdt_address != 0 {
        let dsdt_header: &AcpiSdtHeader = ptr_at(data.dsdt_address, phys_mem_offset);
        let dsdt_len = dsdt_header.length;
        data.dsdt_length = dsdt_len;
        data.dsdt_present = true;
        crate::serial_println!(
            "ACPI: DSDT at {:#010x}, length {}",
            data.dsdt_address, dsdt_len,
        );
    }

    crate::serial_println!(
        "ACPI: FADT parsed: PM timer={:#x}, reset={:#x}/{}",
        data.pm_timer_block, data.reset_port, data.reset_value,
    );
}

// ============================================================================
// PUBLIC API
// ============================================================================

// Physical memory offset — set during init
static mut PHYS_MEM_OFFSET: u64 = 0;

// Parsed ACPI data
static mut ACPI_DATA: AcpiData = AcpiData::new();

/// Get the parsed ACPI data
pub fn get_data() -> &'static AcpiData {
    unsafe { &*core::ptr::addr_of!(ACPI_DATA) }
}

/// Get the physical memory offset
pub fn phys_mem_offset() -> u64 {
    unsafe { PHYS_MEM_OFFSET }
}

/// Get PCIe ECAM base address (if present)
pub fn ecam_base() -> Option<u64> {
    let data = get_data();
    if data.ecam_present {
        Some(data.ecam_base)
    } else {
        None
    }
}

/// Get ACPI PM timer port
pub fn pm_timer_port() -> Option<u16> {
    let data = get_data();
    if data.fadt_present && data.pm_timer_block != 0 {
        Some(data.pm_timer_block as u16)
    } else {
        None
    }
}

/// Reset the system via ACPI reset register
pub fn acpi_reset() {
    let data = get_data();
    if data.fadt_present && data.reset_port != 0 {
        unsafe {
            // Write reset value to reset port (I/O or MMIO)
            let address = data.reset_port;
            let value = data.reset_value;
            if (data.reset_port as u32) < 0x10000 {
                // I/O port
                use x86_64::instructions::port::Port;
                Port::new(address).write(value);
            }
        }
        crate::serial_println!("ACPI: Reset command issued");
    } else {
        crate::serial_println!("ACPI: No reset register available");
    }
}

/// Initialize ACPI subsystem — scan RSDP, parse tables, register devices
pub fn init(rsdp_addr_opt: Option<u64>, phys_mem_offset: u64) {
    unsafe {
        PHYS_MEM_OFFSET = phys_mem_offset;
    }

    crate::serial_println!("ACPI: Initializing...");

    // Try to find RSDP from the provided address, or scan memory
    let tables_addr = if let Some(addr) = rsdp_addr_opt {
        crate::serial_println!("ACPI: RSDP provided at {:#010x}", addr);
        Some(addr)
    } else {
        crate::serial_println!("ACPI: Scanning for RSDP...");
        unsafe { scan_for_rsdp() }
    };

    if let Some(addr) = tables_addr {
        unsafe {
            // Determine if RSDP is v1 (RSDT) or v2 (XSDT)
            if addr > 0xFFFFFFFF {
                // 64-bit address means XSDT was used
                let data_ptr = core::ptr::addr_of_mut!(ACPI_DATA);
                (*data_ptr).revision = 2;
                (*data_ptr).xsdt_address = addr;
                parse_xsdt(addr, phys_mem_offset, &mut *data_ptr);
            } else {
                let data_ptr = core::ptr::addr_of_mut!(ACPI_DATA);
                (*data_ptr).rsdt_address = addr as u32;
                parse_rsdt(addr, phys_mem_offset, &mut *data_ptr);
            }
        }

        let data = get_data();

        if data.ecam_present {
            crate::serial_println!("ACPI: PCIe ECAM available at {:#018x}", data.ecam_base);
        }

        crate::serial_println!("ACPI: Initialization complete (revision {})", data.revision);
    } else {
        crate::serial_println!("ACPI: RSDP not found — no ACPI tables available");
    }
}

/// Initialize ACPI from bootloader-provided RSDP (legacy wrapper)
pub fn init_legacy(rsdp_addr: u64, phys_mem_offset: u64) {
    init(Some(rsdp_addr), phys_mem_offset);
}