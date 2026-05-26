pub mod ethernet;
pub mod mesh;
pub mod onion;
pub mod ssdp;

use crate::{device_manager, println};

pub fn init() {
    println!("Net: Initializing networking stack...");

    // Check if any network device was discovered
    let _devices = device_manager::MANAGER.lock().devices.len();
    let has_net = device_manager::MANAGER
        .lock()
        .devices
        .iter()
        .any(|d| d.class == device_manager::DeviceClass::Network);

    if has_net {
        println!("Net: Network hardware detected. Starting discovery...");
        ethernet::init();
    } else {
        println!("Net: No network hardware found. Networking disabled.");
    }
}

pub async fn discovery_task() {
    // Start SSDP discovery (Implements PERC-003)
    ssdp::discovery_task().await;
}
