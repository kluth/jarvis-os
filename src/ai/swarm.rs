use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spinning_top::Spinlock;

use crate::println;

/// Types of messages exchanged in the swarm.
#[derive(Debug, Clone)]
pub enum SwarmMessage {
    /// Broadcast an intent detected by a local voice shell
    IntentBroadcast {
        intent_id: u64,
        description: String,
        confidence: u8,
    },
    /// Negotiate who handles a specific intent
    TaskNegotiation {
        intent_id: u64,
        bid_score: u8, // Higher score means better suited to handle it
    },
    /// Acknowledge taking over an intent
    TaskAccepted { intent_id: u64 },
    /// Periodic heartbeat with node capabilities
    Heartbeat { capabilities: Vec<String> },
    /// Autonomous system health report
    SystemHealth {
        cpu_load: u8,
        mem_used: usize,
        mem_free: usize,
        uptime_s: u64,
    },
}

impl SwarmMessage {
    /// Serializes the message into a binary format suitable for network transmission.
    /// Following a strict tag-length-value (TLV) or similar compact pattern.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        match self {
            SwarmMessage::IntentBroadcast {
                intent_id,
                description,
                confidence,
            } => {
                buffer.push(0); // Tag
                buffer.extend_from_slice(&intent_id.to_le_bytes());
                buffer.push(*confidence);
                buffer.extend_from_slice(&(description.len() as u32).to_le_bytes());
                buffer.extend_from_slice(description.as_bytes());
            }
            SwarmMessage::TaskNegotiation {
                intent_id,
                bid_score,
            } => {
                buffer.push(1); // Tag
                buffer.extend_from_slice(&intent_id.to_le_bytes());
                buffer.push(*bid_score);
            }
            SwarmMessage::TaskAccepted { intent_id } => {
                buffer.push(2); // Tag
                buffer.extend_from_slice(&intent_id.to_le_bytes());
            }
            SwarmMessage::Heartbeat { capabilities } => {
                buffer.push(3); // Tag
                buffer.extend_from_slice(&(capabilities.len() as u32).to_le_bytes());
                for cap in capabilities {
                    buffer.extend_from_slice(&(cap.len() as u32).to_le_bytes());
                    buffer.extend_from_slice(cap.as_bytes());
                }
            }
            SwarmMessage::SystemHealth {
                cpu_load,
                mem_used,
                mem_free,
                uptime_s,
            } => {
                buffer.push(4); // Tag
                buffer.push(*cpu_load);
                buffer.extend_from_slice(&mem_used.to_le_bytes());
                buffer.extend_from_slice(&mem_free.to_le_bytes());
                buffer.extend_from_slice(&uptime_s.to_le_bytes());
            }
        }
        buffer
    }

    /// Deserializes a binary buffer back into a SwarmMessage.
    pub fn deserialize(data: &[u8]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }
        let tag = data[0];
        let mut cursor = 1;

        match tag {
            0 => {
                // IntentBroadcast
                if data.len() < cursor + 8 + 1 + 4 {
                    return None;
                }
                let intent_id = u64::from_le_bytes(data[cursor..cursor + 8].try_into().ok()?);
                cursor += 8;
                let confidence = data[cursor];
                cursor += 1;
                let len = u32::from_le_bytes(data[cursor..cursor + 4].try_into().ok()?) as usize;
                cursor += 4;
                if data.len() < cursor + len {
                    return None;
                }
                let description = String::from_utf8(data[cursor..cursor + len].to_vec()).ok()?;
                Some(SwarmMessage::IntentBroadcast {
                    intent_id,
                    description,
                    confidence,
                })
            }
            1 => {
                // TaskNegotiation
                if data.len() < cursor + 8 + 1 {
                    return None;
                }
                let intent_id = u64::from_le_bytes(data[cursor..cursor + 8].try_into().ok()?);
                cursor += 8;
                let bid_score = data[cursor];
                Some(SwarmMessage::TaskNegotiation {
                    intent_id,
                    bid_score,
                })
            }
            2 => {
                // TaskAccepted
                if data.len() < cursor + 8 {
                    return None;
                }
                let intent_id = u64::from_le_bytes(data[cursor..cursor + 8].try_into().ok()?);
                Some(SwarmMessage::TaskAccepted { intent_id })
            }
            3 => {
                // Heartbeat
                if data.len() < cursor + 4 {
                    return None;
                }
                let cap_count =
                    u32::from_le_bytes(data[cursor..cursor + 4].try_into().ok()?) as usize;
                cursor += 4;
                let mut capabilities = Vec::new();
                for _ in 0..cap_count {
                    if data.len() < cursor + 4 {
                        return None;
                    }
                    let len =
                        u32::from_le_bytes(data[cursor..cursor + 4].try_into().ok()?) as usize;
                    cursor += 4;
                    if data.len() < cursor + len {
                        return None;
                    }
                    capabilities.push(String::from_utf8(data[cursor..cursor + len].to_vec()).ok()?);
                    cursor += len;
                }
                Some(SwarmMessage::Heartbeat { capabilities })
            }
            4 => {
                // SystemHealth
                if data.len() < cursor + 1 + 8 + 8 + 8 {
                    return None;
                }
                let cpu_load = data[cursor];
                cursor += 1;
                let mem_used = usize::from_le_bytes(data[cursor..cursor + 8].try_into().ok()?);
                cursor += 8;
                let mem_free = usize::from_le_bytes(data[cursor..cursor + 8].try_into().ok()?);
                cursor += 8;
                let uptime_s = u64::from_le_bytes(data[cursor..cursor + 8].try_into().ok()?);
                Some(SwarmMessage::SystemHealth {
                    cpu_load,
                    mem_used,
                    mem_free,
                    uptime_s,
                })
            }
            _ => None,
        }
    }
}

pub struct SwarmAgent {
    node_id: u64,
    active_intents: Spinlock<alloc::collections::BTreeMap<u64, String>>,
}

impl SwarmAgent {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            active_intents: Spinlock::new(alloc::collections::BTreeMap::new()),
        }
    }

    pub fn broadcast_intent(&self, description: &str, confidence: u8) {
        static INTENT_COUNTER: AtomicU64 = AtomicU64::new(1);
        let intent_id = INTENT_COUNTER.fetch_add(1, Ordering::SeqCst);

        let msg = SwarmMessage::IntentBroadcast {
            intent_id,
            description: String::from(description),
            confidence,
        };

        let data = msg.serialize();

        // Dispatch to all peers in the mesh network
        let peers = crate::net::mesh::NODE.get_peer_ids();
        for peer_id in &peers {
            let _ = crate::net::mesh::NODE.send_to(*peer_id, &data);
        }

        println!(
            "Swarm [{}]: Broadcasted intent '{}' to {} peers",
            self.node_id,
            description,
            peers.len()
        );

        self.active_intents
            .lock()
            .insert(intent_id, String::from(description));
    }

    pub fn broadcast_health(&self, cpu_load: u8, mem_used: usize, mem_free: usize, uptime_s: u64) {
        let msg = SwarmMessage::SystemHealth {
            cpu_load,
            mem_used,
            mem_free,
            uptime_s,
        };

        let data = msg.serialize();
        let peers = crate::net::mesh::NODE.get_peer_ids();
        for peer_id in &peers {
            let _ = crate::net::mesh::NODE.send_to(*peer_id, &data);
        }

        println!(
            "[OBS] Swarm [{}]: Distributed health report to {} nodes",
            self.node_id,
            peers.len()
        );
    }

    pub fn handle_message(&self, sender_id: u64, msg: SwarmMessage) {
        match msg {
            SwarmMessage::IntentBroadcast {
                intent_id: _,
                description,
                confidence,
            } => {
                println!(
                    "Swarm [{}]: Received intent '{}' from Node {} (Confidence: {})",
                    self.node_id, description, sender_id, confidence
                );
                // Logic to bid for the task would go here
            }
            SwarmMessage::TaskNegotiation {
                intent_id,
                bid_score,
            } => {
                println!(
                    "Swarm [{}]: Node {} bid {} for intent {}",
                    self.node_id, sender_id, bid_score, intent_id
                );
            }
            SwarmMessage::TaskAccepted { intent_id } => {
                println!(
                    "Swarm [{}]: Node {} accepted intent {}",
                    self.node_id, sender_id, intent_id
                );
                self.active_intents.lock().remove(&intent_id);
            }
            SwarmMessage::Heartbeat { capabilities: _ } => {
                // println!("Swarm [{}]: Heartbeat from Node {} with {} capabilities", self.node_id, sender_id, capabilities.len());
            }
            SwarmMessage::SystemHealth {
                cpu_load,
                mem_used,
                mem_free,
                uptime_s,
            } => {
                println!(
                    "Swarm [{}]: Remote health from Node {} (CPU: {}%, Mem: {}/{} bytes, Uptime: {}s)",
                    self.node_id,
                    sender_id,
                    cpu_load,
                    mem_used,
                    mem_used + mem_free,
                    uptime_s
                );
            }
        }
    }
}

lazy_static! {
    pub static ref AGENT: Arc<SwarmAgent> = Arc::new(SwarmAgent::new(1));
}

/// Background task to handle swarm coordination and message processing.
pub async fn swarm_task() {
    println!("Swarm: Multi-agent orchestration engine online.");

    loop {
        // 1. Process incoming secure packets from the mesh network
        while let Some(packet) = crate::net::mesh::NODE.receive_next() {
            if let Some(msg) = SwarmMessage::deserialize(&packet.payload) {
                AGENT.handle_message(packet.sender_id, msg);
            }
        }

        // 2. Perform periodic autonomic heartbeat
        let capabilities = alloc::vec![String::from("vision"), String::from("computation")];
        let heartbeat = SwarmMessage::Heartbeat { capabilities };
        let data = heartbeat.serialize();

        let peers = crate::net::mesh::NODE.get_peer_ids();
        for peer_id in &peers {
            let _ = crate::net::mesh::NODE.send_to(*peer_id, &data);
        }

        crate::task::yield_now().await;
    }
}

#[cfg(feature = "test")]
pub fn test_swarm_logic() {
    crate::serial_print!("test_swarm_logic... ");
    let agent = SwarmAgent::new(99);
    agent.broadcast_intent("Turn on lights", 95);

    let active = agent.active_intents.lock();
    assert_eq!(active.len(), 1);
    assert_eq!(active.values().next().unwrap(), "Turn on lights");

    crate::serial_println!("[ok]");
}
