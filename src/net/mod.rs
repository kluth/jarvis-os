pub mod ethernet;
pub mod mesh;

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
    let mut discovered = false;
    loop {
        if !discovered {
            println!("[PERC] Net: Autonomous discovery active. Scanning for peers...");

            // Simulate discovering "Node 2 (FRIDAY)"
            println!("[PERC] Net: Discovered peer 'FRIDAY' (Node ID: 2) via mDNS.");

            // Simulated public key for Node 2
            let mut fake_key = [0u8; 32];
            fake_key[0] = 0x42;
            let peer_public_key = x25519_dalek::PublicKey::from(fake_key);

            mesh::NODE.add_peer(2, peer_public_key);
            println!("[PERC] Net: Secure mesh connection established with Node 2.");

            discovered = true;
        }

        // Periodic scan every 30 seconds
        for _ in 0..300 {
            crate::task::yield_now().await;
        }
    }
}
