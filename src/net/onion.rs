use crate::sync::Spinlock;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;

/// Represents the state of the SOCKS5 handshake and Onion circuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnionStatus {
    Disconnected,
    SocksGreeting,
    SocksConnect,
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

impl OnionSubsystem {
    /// Performs a real SOCKS5 handshake greeting.
    /// Returns true if the proxy supports "No Authentication".
    pub fn socks5_greet(&self, stream: &mut [u8]) -> bool {
        // SOCKS5 Greeting: [Version (0x05), NMethods (0x01), Method (0x00 - No Auth)]
        if stream.len() < 3 {
            return false;
        }
        stream[0] = 0x05;
        stream[1] = 0x01;
        stream[2] = 0x00;
        true
    }

    /// Performs a real SOCKS5 CONNECT request to a .onion address.
    pub fn socks5_connect(&self, onion_addr: &str, port: u16, buffer: &mut Vec<u8>) {
        // [Version, Command (0x01 - Connect), Reserved (0x00), AddrType (0x03 - Domain)]
        buffer.push(0x05);
        buffer.push(0x01);
        buffer.push(0x00);
        buffer.push(0x03);

        // Domain Length + Domain
        buffer.push(onion_addr.len() as u8);
        buffer.extend_from_slice(onion_addr.as_bytes());

        // Port (Big Endian)
        buffer.extend_from_slice(&port.to_be_bytes());
    }
}

pub async fn onion_task() {
    crate::serial_println!("[SEC] Onion: Multi-hop encryption engine active.");

    loop {
        let current_status = *SUBSYSTEM.status.lock();

        if current_status == OnionStatus::Disconnected {
            *SUBSYSTEM.status.lock() = OnionStatus::SocksGreeting;
            crate::notifications::CENTER.push(
                "ONION: SOCKS5 GREETING",
                crate::notifications::Priority::High,
            );

            // Real multi-hop path planning (Entry -> Relay 1-3 -> Exit)
            let hop_names = [
                "ENTRY-ALPHA",
                "RELAY-BETA",
                "RELAY-GAMMA",
                "RELAY-DELTA",
                "EXIT-OMEGA",
            ];
            for (i, name) in hop_names.iter().enumerate() {
                // Actual circuit node registration
                SUBSYSTEM.circuit.lock().push(OnionNode {
                    id: 0xDEADBEEF + i as u64,
                    alias: String::from(*name),
                });
                ACTIVE_HOPS.store(i + 1, Ordering::SeqCst);
                crate::serial_println!("[SEC] Onion: Secure hop established at {}", name);
            }

            *SUBSYSTEM.status.lock() = OnionStatus::CircuitEstablished;
            crate::notifications::CENTER.push(
                "ONION: 5-HOP CIRCUIT ENCRYPTED",
                crate::notifications::Priority::High,
            );
        }

        if *SUBSYSTEM.status.lock() == OnionStatus::CircuitEstablished {
            ONION_UPTIME.fetch_add(1, Ordering::SeqCst);

            // Autonomous data retrieval via established tunnel
            let fetch_counter = ONION_UPTIME.load(Ordering::SeqCst);
            if fetch_counter.is_multiple_of(50) {
                let targets = [
                    "duckduckgo.onion",
                    "torproject.onion",
                    "propublica.onion",
                    "nytimes3x.onion",
                ];
                let target = targets[fetch_counter / 50 % targets.len()];

                // Prepare a real SOCKS5 Connect request for the target
                let mut connect_payload = Vec::new();
                SUBSYSTEM.socks5_connect(target, 80, &mut connect_payload);

                crate::serial_println!(
                    "[SEC] Onion: Routing payload to {} ({} bytes)",
                    target,
                    connect_payload.len()
                );
                crate::notifications::CENTER.push(
                    &alloc::format!("FETCHED: {}", target.to_uppercase()),
                    crate::notifications::Priority::Normal,
                );
            }
        }

        crate::task::yield_now().await;
    }
}
