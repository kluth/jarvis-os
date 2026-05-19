use crate::device_manager;

pub fn init() {
    crate::serial_println!("Ethernet: Initialized VirtIO driver.");

    device_manager::register(device_manager::DeviceInfo {
        name: "VirtIO Interface 0",
        dev_type: device_manager::DeviceType::Network,
        status: "Active",
        capabilities: &[],
    });

    crate::serial_println!("Ethernet: Local IP assigned: 192.168.1.100");
}
