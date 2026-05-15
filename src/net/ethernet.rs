use crate::{println, device_manager};
use alloc::string::ToString;

pub fn init() {
    println!("Ethernet: Initialized VirtIO driver.");
    
    device_manager::register(device_manager::DeviceInfo {
        name: "VirtIO Interface 0".to_string(),
        dev_type: device_manager::DeviceType::Network,
        status: "DHCP Requesting...",
    });
    
    println!("Ethernet: Local IP assigned: 192.168.1.100 (simulated)");
}
