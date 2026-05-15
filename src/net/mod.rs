pub mod ethernet;

use crate::{device_manager, println};

pub fn init() {
    println!("Net: Initializing networking stack...");

    // Check if any network device was discovered
    let devices = device_manager::MANAGER.lock().get_devices().clone();
    let has_net = devices
        .iter()
        .any(|d| matches!(d.dev_type, device_manager::DeviceType::Network));

    if has_net {
        println!("Net: Network hardware detected. Starting discovery...");
        ethernet::init();
    } else {
        println!("Net: No network hardware found. Networking disabled.");
    }
}

pub async fn discovery_task() {
    loop {
        // Periodically perform service discovery
        // println!("Net: Scanning for Stark Industries services...");
        core::future::ready(()).await;
    }
}
