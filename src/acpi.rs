use crate::{device_manager, println};
use x86_64::PhysAddr;

pub fn init(rsdp_addr: PhysAddr) {
    println!("ACPI: Initializing at PhysAddr({:?})", rsdp_addr);

    // In a real implementation, we would map the RSDP and parse it.
    // For now, we simulate the discovery of system capabilities.

    device_manager::register(device_manager::DeviceInfo {
        name: "ACPI Power Controller",
        dev_type: device_manager::DeviceType::System,
        status: "Enabled",
    });

    device_manager::register(device_manager::DeviceInfo {
        name: "IO APIC",
        dev_type: device_manager::DeviceType::System,
        status: "Enabled",
    });
}
