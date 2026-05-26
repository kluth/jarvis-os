//! ============================================================================
//! JARVIS OS PCI / PCIe Subsystem — Enhanced Device Discovery
//! ============================================================================
//! Implements:
//!   - Legacy PCI config space via CF8/CFC port I/O
//!   - PCIe ECAM (Memory-Mapped Config Space) via MCFG from ACPI
//!   - PCI capability parser (Power Management, MSI, MSI-X, PCIe, VPD)
//!   - MSI/MSI-X message address/data programming
//!   - Full device enumeration with BAR decoding (I/O, MMIO32, MMIO64)
//! ============================================================================

use crate::device_manager::{Device, DeviceClass, MmioRegion, IrqType};
use alloc::vec::Vec;
use x86_64::instructions::port::Port;

// ============================================================================
// PCI CONFIG SPACE ACCESS MODE
// ============================================================================

enum PciAccess {
    Legacy,                          // CF8/CFC port I/O
    Ecam { base: u64, end_bus: u8 }, // Memory-mapped via MCFG
}

// Auto-selected access method
static mut ACCESS_MODE: Option<PciAccess> = None;

/// Configure PCI access mode (call once ACPI MCFG is parsed)
pub fn set_ecam(base: u64, end_bus: u8) {
    unsafe {
        ACCESS_MODE = Some(PciAccess::Ecam { base, end_bus });
    }
    crate::serial_println!("PCI: ECAM enabled at {:#018x}, bus 0-{}", base, end_bus);
}

fn use_ecam() -> bool {
    unsafe {
        matches!(ACCESS_MODE, Some(PciAccess::Ecam { .. }))
    }
}

// ============================================================================
// PCI CONFIG READ/WRITE
// ============================================================================

/// Read 32-bit from PCI config space (auto ECAM or legacy)
pub fn pci_read_config(bus: u8, slot: u8, func: u8, offset: u16) -> u32 {
    unsafe {
        match ACCESS_MODE {
            Some(PciAccess::Ecam { base, end_bus }) if bus <= end_bus => {
                // ECAM: MMIO at base + ((bus << 20) | (slot << 15) | (func << 12) | offset)
                let phys_addr = base
                    + ((bus as u64) << 20)
                    + ((slot as u64) << 15)
                    + ((func as u64) << 12)
                    + (offset as u64 & 0xFFC);
                let virt = (crate::acpi::phys_mem_offset() + phys_addr) as *const u32;
                core::ptr::read_volatile(virt)
            }
            _ => {
                // Legacy CF8/CFC
                let addr = 0x80000000u32
                    | ((bus as u32) << 16)
                    | ((slot as u32) << 11)
                    | ((func as u32) << 8)
                    | ((offset as u32) & 0xFC);
                let mut cfg_addr = Port::<u32>::new(0xCF8);
                let mut cfg_data = Port::<u32>::new(0xCFC);
                cfg_addr.write(addr);
                cfg_data.read()
            }
        }
    }
}

/// Write 32-bit to PCI config space
pub fn pci_write_config(bus: u8, slot: u8, func: u8, offset: u16, value: u32) {
    unsafe {
        match ACCESS_MODE {
            Some(PciAccess::Ecam { base, end_bus }) if bus <= end_bus => {
                let phys_addr = base
                    + ((bus as u64) << 20)
                    + ((slot as u64) << 15)
                    + ((func as u64) << 12)
                    + (offset as u64 & 0xFFC);
                let virt = (crate::acpi::phys_mem_offset() + phys_addr) as *mut u32;
                core::ptr::write_volatile(virt, value);
            }
            _ => {
                let addr = 0x80000000u32
                    | ((bus as u32) << 16)
                    | ((slot as u32) << 11)
                    | ((func as u32) << 8)
                    | ((offset as u32) & 0xFC);
                let mut cfg_addr = Port::<u32>::new(0xCF8);
                let mut cfg_data = Port::<u32>::new(0xCFC);
                cfg_addr.write(addr);
                let orig = cfg_data.read();
                let mask = match offset & 3 {
                    0 => 0xFFFFFFFFu32,
                    _ => !(0xFFFFFFFFu32 << ((offset & 3) * 8)),
                };
                cfg_data.write((orig & mask) | value);
            }
        }
    }
}

/// Read 16-bit from PCI config
pub fn pci_read_word(bus: u8, slot: u8, func: u8, offset: u16) -> u16 {
    let aligned = offset & 0xFC;
    let shift = ((offset & 0x03) * 8) as u32;
    (pci_read_config(bus, slot, func, aligned) >> shift) as u16
}

/// Write 16-bit to PCI config
pub fn pci_write_word(bus: u8, slot: u8, func: u8, offset: u16, value: u16) {
    let aligned = offset & 0xFC;
    let shift = ((offset & 0x03) * 8) as u32;
    let orig = pci_read_config(bus, slot, func, aligned);
    let mask = (0xFFFFu32) << shift;
    pci_write_config(bus, slot, func, aligned, (orig & !mask) | ((value as u32) << shift));
}

/// Read 8-bit from PCI config
pub fn pci_read_byte(bus: u8, slot: u8, func: u8, offset: u16) -> u8 {
    (pci_read_word(bus, slot, func, offset) & 0xFF) as u8
}

// ============================================================================
// PCI CAPABILITY LIST PARSER
// ============================================================================

/// Standard PCI capability IDs
pub const CAP_PM: u8 = 0x01;      // Power Management
pub const CAP_MSI: u8 = 0x05;     // MSI (Message Signaled Interrupts)
pub const CAP_PCIEXPRESS: u8 = 0x10; // PCI Express Capability
pub const CAP_MSIX: u8 = 0x11;    // MSI-X

/// Parse the capabilities list for a PCI function
/// Returns Vec of (cap_id, cap_offset)
pub fn read_capabilities(bus: u8, slot: u8, func: u8) -> Vec<(u8, u16)> {
    let mut caps = Vec::new();

    // Check if this function has a capabilities list
    let status = pci_read_word(bus, slot, func, 0x06);
    if status & (1 << 4) == 0 {
        return caps; // No capabilities
    }

    let mut offset = (pci_read_byte(bus, slot, func, 0x34) & 0xFC) as u16;
    while offset != 0 && offset != 0xFF {
        let cap_id = pci_read_byte(bus, slot, func, offset);
        let next = pci_read_byte(bus, slot, func, offset + 1) as u16 & 0xFC;
        caps.push((cap_id, offset));
        offset = next;
    }

    caps
}

// ============================================================================
// MSI/MSI-X PROGRAMMING
// ============================================================================

/// Program a device's MSI capability with a given address and data
pub fn program_msi(bus: u8, slot: u8, func: u8, caps: &[(u8, u16)],
                   msi_address: u64, msi_data: u16) -> bool {
    for &(id, offset) in caps {
        if id == CAP_MSI {
            // Read MSI capability register
            let mut msg_ctrl = pci_read_word(bus, slot, func, offset + 2);

            // Determine MSI format (32-bit or 64-bit address)
            let is_64bit = (msg_ctrl >> 7) & 1 != 0;

            // Enable MSI
            msg_ctrl |= 1; // MSI Enable

            // Write message address
            if is_64bit {
                pci_write_config(bus, slot, func, offset + 4, msi_address as u32);
                pci_write_config(bus, slot, func, offset + 8, (msi_address >> 32) as u32);
                pci_write_word(bus, slot, func, offset + 0x0C, msi_data);
                // Write message control (with enable bit)
                pci_write_word(bus, slot, func, offset + 2, msg_ctrl);
            } else {
                pci_write_config(bus, slot, func, offset + 4, msi_address as u32);
                pci_write_word(bus, slot, func, offset + 0x08, msi_data);
                pci_write_word(bus, slot, func, offset + 2, msg_ctrl);
            }

            crate::serial_println!("PCI: MSI programmed on [{}:{}:{}]", bus, slot, func);
            return true;
        }
    }
    false
}

// ============================================================================
// BAR DECODING
// ============================================================================

/// Read and decode all 6 BARs for a PCI function
pub fn read_bars(bus: u8, slot: u8, func: u8) -> [MmioRegion; 6] {
    let mut bars = [MmioRegion::empty(); 6];
    let mut bar64_high = false;

    for i in 0..6 {
        if bar64_high {
            // This is the high dword of a 64-bit BAR — skip
            bar64_high = false;
            continue;
        }

        let offset = 0x10 + (i as u16 * 4);
        let bar_raw = pci_read_config(bus, slot, func, offset);
        if bar_raw == 0 {
            continue; // Unimplemented BAR
        }

        // Determine BAR type
        if bar_raw & 1 != 0 {
            // I/O BAR
            let base = (bar_raw & 0xFFFFFFFC) as u64;
            bars[i] = MmioRegion {
                base,
                len: 256,
                prefetchable: false,
                is_mmio: false,
            };
        } else {
            // MMIO BAR
            let mmio_type = (bar_raw >> 1) & 0x03;
            match mmio_type {
                0x00 => {
                    // 32-bit MMIO
                    let base = (bar_raw & 0xFFFFFFF0) as u64;
                    // Read BAR size by writing all 1s
                    pci_write_config(bus, slot, func, offset, 0xFFFFFFFF);
                    let size_raw = pci_read_config(bus, slot, func, offset);
                    pci_write_config(bus, slot, func, offset, bar_raw); // Restore
                    let size = (!(size_raw & 0xFFFFFFF0) + 1) as usize;

                    bars[i] = MmioRegion {
                        base,
                        len: size,
                        prefetchable: (bar_raw >> 3) & 1 != 0,
                        is_mmio: true,
                    };
                }
                0x02 => {
                    // 64-bit MMIO (uses two BAR slots)
                    let base_low = bar_raw & 0xFFFFFFF0;
                    let base_high = pci_read_config(bus, slot, func, offset + 4);
                    let base = (base_low as u64) | ((base_high as u64) << 32);

                    // Read size
                    pci_write_config(bus, slot, func, offset, 0xFFFFFFFF);
                    pci_write_config(bus, slot, func, offset + 4, 0xFFFFFFFF);
                    let size_raw_low = pci_read_config(bus, slot, func, offset);
                    let size_raw_high = pci_read_config(bus, slot, func, offset + 4);
                    pci_write_config(bus, slot, func, offset, bar_raw);
                    pci_write_config(bus, slot, func, offset + 4, base_high);

                    let size_raw = (size_raw_low as u64) | ((size_raw_high as u64) << 32);
                    let size = (!(size_raw & 0xFFFFFFF0_FFFFFFF0) + 1) as usize;

                    bars[i] = MmioRegion {
                        base,
                        len: size,
                        prefetchable: (bar_raw >> 3) & 1 != 0,
                        is_mmio: true,
                    };
                    bar64_high = true; // Skip next BAR slot
                }
                _ => {}
            }
        }
    }

    bars
}

// ============================================================================
// DEVICE DISCOVERY — Full PCI/PCIe bus scan
// ============================================================================

/// Information about a discovered PCI function
#[derive(Debug, Clone)]
pub struct PciFunctionInfo {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision: u8,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub subsystem_vendor: u16,
    pub subsystem_device: u16,
    pub bars: [MmioRegion; 6],
    pub irq_pin: u8,
    pub irq_line: u8,
    pub is_pci_pci_bridge: bool,
    pub secondary_bus: u8,
    pub capabilities: Vec<(u8, u16)>,
}

impl PciFunctionInfo {
    pub fn device_class(&self) -> DeviceClass {
        DeviceClass::from_pci(self.class, self.subclass)
    }
}

/// Check if a vendor/device pair indicates a valid device
fn is_valid_device(vendor: u16, _device: u16) -> bool {
    vendor != 0xFFFF && vendor != 0x0000
}

/// Scan a single PCI function
fn scan_function(bus: u8, slot: u8, func: u8, _legacy: bool) -> Option<PciFunctionInfo> {
    let vendor = pci_read_word(bus, slot, func, 0);
    if vendor == 0xFFFF {
        return None;
    }

    let device = pci_read_word(bus, slot, func, 2);
    if !is_valid_device(vendor, device) {
        return None;
    }

    let class_rev = pci_read_config(bus, slot, func, 0x08);
    let revision = (class_rev & 0xFF) as u8;
    let prog_if = ((class_rev >> 8) & 0xFF) as u8;
    let subclass = ((class_rev >> 16) & 0xFF) as u8;
    let class = (class_rev >> 24) as u8;

    let subsys = pci_read_config(bus, slot, func, 0x2C);
    let subsystem_vendor = (subsys & 0xFFFF) as u16;
    let subsystem_device = (subsys >> 16) as u16;

    let irq_info = pci_read_config(bus, slot, func, 0x3C);
    let irq_line = (irq_info & 0xFF) as u8;
    let irq_pin = ((irq_info >> 8) & 0xFF) as u8;

    let bars = read_bars(bus, slot, func);
    let caps = read_capabilities(bus, slot, func);

    // Check if this is a PCI-PCI bridge
    let is_pci_pci_bridge = class == 0x06 && subclass == 0x04;
    let secondary_bus = if is_pci_pci_bridge {
        pci_read_byte(bus, slot, func, 0x19)
    } else {
        0
    };

    Some(PciFunctionInfo {
        bus, slot, func,
        vendor_id: vendor,
        device_id: device,
        revision, class, subclass, prog_if,
        subsystem_vendor, subsystem_device,
        bars, irq_pin, irq_line,
        is_pci_pci_bridge, secondary_bus,
        capabilities: caps,
    })
}

/// Scan a single PCI slot (all functions)
fn scan_slot(bus: u8, slot: u8, legacy: bool, functions: &mut Vec<PciFunctionInfo>) {
    // Check function 0
    if let Some(info) = scan_function(bus, slot, 0, legacy) {
        functions.push(info);

        // Check if this is a multi-function device
        let header_type = pci_read_byte(bus, slot, 0, 0x0E);
        if header_type & 0x80 != 0 {
            // Multi-function — scan functions 1-7
            for func in 1..=7u8 {
                if let Some(info) = scan_function(bus, slot, func, legacy) {
                    functions.push(info);
                }
            }
        }
    }
}

/// Scan an entire PCI bus
fn scan_bus(bus: u8, legacy: bool, functions: &mut Vec<PciFunctionInfo>) {
    for slot in 0..=31u8 {
        scan_slot(bus, slot, legacy, functions);
    }
}

/// Perform a complete PCI/PCIe bus scan, including bridge traversal
pub fn scan_all_buses() -> Vec<PciFunctionInfo> {
    let mut functions = Vec::new();

    // Determine if we can use legacy CF8 or need ECAM
    let is_legacy = !use_ecam();

    // Use a worklist to scan buses recursively (handles nested bridges)
    let mut buses_to_scan = Vec::new();
    buses_to_scan.push(0);

    let mut scanned_buses = [false; 256];
    scanned_buses[0] = true;

    while let Some(bus) = buses_to_scan.pop() {
        let mut bus_functions = Vec::new();
        scan_bus(bus, is_legacy, &mut bus_functions);
        
        for f in &bus_functions {
            if f.is_pci_pci_bridge && f.secondary_bus != 0 {
                let sec_bus = f.secondary_bus;
                if !scanned_buses[sec_bus as usize] {
                    buses_to_scan.push(sec_bus);
                    scanned_buses[sec_bus as usize] = true;
                }
            }
        }
        functions.extend(bus_functions);
    }

    // If ECAM is available, scan any remaining buses up to end_bus (fills gaps)
    let ecam_end_bus = {
        let data = crate::acpi::get_data();
        if data.ecam_present { Some(data.ecam_end_bus) } else { None }
    };
    if let Some(end_bus) = ecam_end_bus {
        for bus in 0..=end_bus {
            if !scanned_buses[bus as usize] {
                scan_bus(bus, is_legacy, &mut functions);
                scanned_buses[bus as usize] = true;
            }
        }
    }

    functions
}

// ============================================================================
// CONVENIENCE: Scan and return display controller info
// ============================================================================

pub fn pci_function_to_device(info: &PciFunctionInfo) -> Device {
    let class = DeviceClass::from_pci(info.class, info.subclass);
    let name: &'static str = match (info.class, info.subclass) {
        // Mass storage
        (0x01, 0x01) => "IDE Controller",
        (0x01, 0x06) => "SATA AHCI Controller",
        (0x01, 0x08) => "NVMe Controller",
        // Network
        (0x02, 0x00) => "Ethernet Controller",
        (0x02, 0x01) => "Token Ring Controller",
        (0x02, 0x80) => "Network Controller",
        // Display
        (0x03, 0x00) => "VGA/Display Controller",
        (0x03, 0x01) => "XGA Controller",
        (0x03, 0x02) => "3D Controller",
        // Multimedia
        (0x04, 0x00) => "Multimedia Video",
        (0x04, 0x01) => "Multimedia Audio",
        (0x04, 0x03) => "High Definition Audio",
        // Bridge
        (0x06, 0x00) => "Host Bridge",
        (0x06, 0x01) => "ISA Bridge",
        (0x06, 0x04) => "PCI-PCI Bridge",
        (0x06, 0x80) => "Bridge",
        // Communication
        (0x07, 0x00) => "Serial Controller",
        (0x07, 0x01) => "Parallel Controller",
        // Base System Peripherals
        (0x08, 0x00) => "Interrupt Controller",
        (0x08, 0x01) => "DMA Controller",
        (0x08, 0x02) => "Timer",
        (0x08, 0x03) => "RTC Controller",
        // Input
        (0x09, 0x00) => "Keyboard Controller",
        (0x09, 0x01) => "Digitizer",
        (0x09, 0x02) => "Mouse Controller",
        // Serial Bus
        (0x0C, 0x03) => {
            match info.prog_if {
                0x00 => "USB UHCI Controller",
                0x10 => "USB OHCI Controller",
                0x20 => "USB EHCI Controller",
                0x30 => "USB xHCI Controller",
                0x80 => "USB Controller",
                0xFE => "USB Device",
                _ => "USB Controller",
            }
        }
        (0x0C, 0x05) => "SMBus Controller",
        // Encryption
        (0x0D, 0x00) => "Network/Computing Encryption",
        (0x0D, 0x10) => "Entertainment Encryption",
        // Generic
        _ => "PCI Device",
    };

    let mut device = Device::new(info.vendor_id, info.device_id, class, info.bus, info.slot, info.func);
    device.name = name;
    device.id.subsystem_vendor = info.subsystem_vendor;
    device.id.subsystem_device = info.subsystem_device;
    device.id.revision = info.revision;
    device.bars = info.bars;

    if info.irq_line != 0 {
        device.irq = info.irq_line;
        device.irq_type = IrqType::Legacy;

        // Check for MSI capability
        for &(cap_id, _) in &info.capabilities {
            if cap_id == CAP_MSI {
                device.irq_type = IrqType::Msi;
                break;
            }
            if cap_id == CAP_MSIX {
                device.irq_type = IrqType::Msix;
            }
        }
    }

    device
}

/// Configures MSI for a device
pub fn configure_msi(bus: u8, slot: u8, func: u8, vector: u8, cpu_id: u8) -> Result<(), &'static str> {
    // Find MSI capability offset
    let mut cap_ptr = pci_read_word(bus, slot, func, 0x34) & 0xFF;
    let mut msi_off = 0;
    
    while cap_ptr != 0 {
        let cap_id = pci_read_word(bus, slot, func, cap_ptr) & 0xFF;
        if cap_id == 0x05 { // MSI
            msi_off = cap_ptr;
            break;
        }
        cap_ptr = (pci_read_word(bus, slot, func, cap_ptr) >> 8) & 0xFF;
    }

    if msi_off == 0 {
        return Err("MSI capability not found");
    }

    // Configure MSI
    // Message Address: 0xFEE00000 | (cpu_id << 12)
    let addr = 0xFEE00000 | ((cpu_id as u32) << 12);
    pci_write_config(bus, slot, func, ((msi_off + 4)), addr);

    // Message Data: vector
    pci_write_config(bus, slot, func, ((msi_off + 8)), vector as u32);

    // Enable MSI: set bit 16 of Message Control
    let mut ctrl = pci_read_word(bus, slot, func, ((msi_off + 2)));
    ctrl |= 0x0001;
    pci_write_word(bus, slot, func, ((msi_off + 2)), ctrl);

    Ok(())
}

// ============================================================================
// TOP-LEVEL INIT
// ============================================================================

/// Initialize PCI subsystem: scan all buses, register all devices
pub fn init() {
    crate::serial_println!("PCI: Initializing...");

    // Try to enable ECAM from ACPI
    if let Some(base) = crate::acpi::ecam_base() {
        let end_bus = crate::acpi::get_data().ecam_end_bus;
        set_ecam(base, end_bus);
    }

    let functions = scan_all_buses();
    crate::serial_println!("PCI: Found {} function(s) on {} bus(es)",
        functions.len(),
        functions.iter().map(|f| f.bus).max().unwrap_or(0) + 1,
    );

    // Register each function with Device Manager
    for info in &functions {
        let dev = pci_function_to_device(info);
        crate::device_manager::register_device(dev);
    }

    // Enable Bus Mastering + Memory Space for display devices
    for info in &functions {
        if info.class == 0x03 {
            let bus = info.bus;
            let slot = info.slot;
            let func = info.func;
            let mut cmd = pci_read_word(bus, slot, func, 0x04);
            cmd |= 0x07; // I/O Space | Memory Space | Bus Master
            pci_write_word(bus, slot, func, 0x04, cmd);
            crate::serial_println!(
                "PCI: Enabled bus mastering on [{}:{}.{}] {}",
                bus, slot, func,
                pci_function_to_device(info).name,
            );
        }
    }

    crate::serial_println!("PCI: Initialization complete");
}

// ============================================================================
// CONVENIENCE: Scan and return display controller info
// ============================================================================

pub fn find_display_devices() -> Vec<PciFunctionInfo> {
    let mut displays = Vec::new();
    let functions = scan_all_buses();
    for f in &functions {
        if f.class == 0x03 {
            displays.push(f.clone());
        }
    }
    displays
}

// ============================================================================
// BACKWARD COMPATIBILITY — existing code uses these
// ============================================================================

/// Legacy scan_bus() for existing callers
pub fn scan_bus_legacy() {
    init();
}

pub fn pci_read_word_compat(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    pci_read_word(bus, slot, func, offset as u16)
}

pub fn pci_write_word_compat(bus: u8, slot: u8, func: u8, offset: u8, value: u16) {
    pci_write_word(bus, slot, func, offset as u16, value)
}

pub fn pci_read_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    pci_read_config(bus, slot, func, offset as u16)
}

pub fn pci_write_dword(bus: u8, slot: u8, func: u8, offset: u8, value: u32) {
    pci_write_config(bus, slot, func, offset as u16, value)
}
