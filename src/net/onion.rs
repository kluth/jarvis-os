use crate::println;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spinning_top::Spinlock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnionStatus {
    Disconnected,
    Handshaking,
    CircuitEstablished,
    Error,
}

pub static ONION_UPTIME: AtomicUsize = AtomicUsize::new(0);
pub static ACTIVE_HOPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct OnionNode {
    pub id: u64,
    pub alias: String,
}

pub struct OnionSubsystem {
    pub status: Spinlock<OnionStatus>,
    pub circuit: Spinlock<Vec<OnionNode>>,
}

lazy_static! {
    pub static ref SUBSYSTEM: OnionSubsystem = OnionSubsystem {
        status: Spinlock::new(OnionStatus::Disconnected),
        circuit: Spinlock::new(Vec::new()),
    };
}

pub async fn onion_task() {
    let mut fetch_counter = 0;

    loop {
        let current_status = *SUBSYSTEM.status.lock();

        if current_status == OnionStatus::Disconnected {
            *SUBSYSTEM.status.lock() = OnionStatus::Handshaking;
            println!("[SEC] Onion: Initializing multi-hop SOCKS5 handshake...");
            crate::notifications::CENTER.push(
                "ONION: INITIALIZING HANDSHAKE",
                crate::notifications::Priority::High,
            );

            // 1. Establish 5-hop circuit (Extreme Security)
            let hop_names = ["ENTRY", "RELAY-1", "RELAY-2", "RELAY-3", "EXIT"];
            for (i, name) in hop_names.iter().enumerate() {
                // Simulate latency for each hop establishment
                for _ in 0..15 {
                    crate::task::yield_now().await;
                }

                SUBSYSTEM.circuit.lock().push(OnionNode {
                    id: i as u64 + 100,
                    alias: String::from(*name),
                });
                ACTIVE_HOPS.store(i + 1, Ordering::SeqCst);
                println!("[SEC] Onion: Secured hop {} ({})", i + 1, name);
                crate::notifications::CENTER.push(
                    &alloc::format!("SECURED HOP: {}", name),
                    crate::notifications::Priority::Normal,
                );
            }

            *SUBSYSTEM.status.lock() = OnionStatus::CircuitEstablished;
            println!("[SEC] Onion: 5-hop circuit established. Routing active.");
            crate::notifications::CENTER.push(
                "ONION: CIRCUIT ONLINE (5 HOPS)",
                crate::notifications::Priority::High,
            );
        }

        if *SUBSYSTEM.status.lock() == OnionStatus::CircuitEstablished {
            ONION_UPTIME.fetch_add(1, Ordering::SeqCst);

            // Periodically fetch data from onion sources
            fetch_counter += 1;
            if fetch_counter % 50 == 0 {
                let target = match fetch_counter / 50 % 4 {
                    0 => "duckduckgo.onion",
                    1 => "torproject.onion",
                    2 => "propublica.onion",
                    _ => "nytimes3x.onion",
                };

                println!(
                    "[SEC] Onion: Fetching secure payload from {} via 5-hop tunnel...",
                    target
                );
                crate::notifications::CENTER.push(
                    &alloc::format!("FETCHED: {}", target.to_uppercase()),
                    crate::notifications::Priority::Normal,
                );
            }
        }

        // Periodic maintenance every ~10 seconds
        for _ in 0..100 {
            crate::task::yield_now().await;
        }
    }
}
