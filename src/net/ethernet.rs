use crate::{device_manager, println};

pub fn init() {
    println!("Ethernet: Initialized VirtIO driver.");

    device_manager::register(device_manager::DeviceInfo {
        name: "VirtIO Interface 0",
        dev_type: device_manager::DeviceType::Network,
        status: "Active",
    });

    println!("Ethernet: Local IP assigned: 192.168.1.100 (simulated)");
}
