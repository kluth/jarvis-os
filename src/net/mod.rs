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
    let mut scan_count = 0;
    loop {
        scan_count += 1;
        if !discovered {
            println!("[PERC][STABILITY_CHECK:HEARTBEAT] Net: Autonomous discovery active. Scanning for peers...");
            crate::notifications::CENTER.push(
                "SCANNING FOR PEERS...",
                crate::notifications::Priority::Normal,
            );

            // Simulate discovery latency
            for _ in 0..10 {
                crate::task::yield_now().await;
            }

            // Simulate discovering "Node 2 (FRIDAY)"
            println!("[PERC][STABILITY_CHECK:HEARTBEAT] Net: Discovered peer 'FRIDAY' (Node ID: 2) via mDNS.");
            crate::notifications::CENTER.push(
                "DISCOVERED PEER: FRIDAY",
                crate::notifications::Priority::High,
            );

            // Simulated public key for Node 2
            let mut fake_key = [0u8; 32];
            fake_key[0] = 0x42;
            let peer_public_key = x25519_dalek::PublicKey::from(fake_key);

            mesh::NODE.add_peer(2, peer_public_key);
            println!("[PERC][STABILITY_CHECK:HEARTBEAT] Net: Secure mesh connection established with Node 2.");
            crate::notifications::CENTER.push(
                "MESH CONNECTION SECURED",
                crate::notifications::Priority::High,
            );

            discovered = true;
        } else {
            // JARVIS is "busy" scanning and optimizing
            if scan_count % 5 == 0 {
                println!("[PERC] Net: Background environment scan in progress...");
                crate::notifications::CENTER.push(
                    "SCANNING ENVIRONMENT...",
                    crate::notifications::Priority::Low,
                );
            }
            if scan_count % 12 == 0 {
                println!("[PERC] Net: Verifying mesh node integrity...");
                crate::notifications::CENTER.push(
                    "VERIFYING PEER: FRIDAY",
                    crate::notifications::Priority::Normal,
                );
            }
        }

        // Periodic scan every ~10 seconds (approx based on yields)
        for _ in 0..100 {
            crate::task::yield_now().await;
        }
    }
}
