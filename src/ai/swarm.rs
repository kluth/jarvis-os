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
    Heartbeat { _capabilities: Vec<String> },
}

impl SwarmMessage {
    // Basic serialization for the prototype. In production, use serde/bincode.
    pub fn serialize(&self) -> Vec<u8> {
        // Dummy serialization: just returning an empty vec for the prototype
        // to avoid complex no_std serialization boilerplate in this step.
        alloc::vec![]
    }

    pub fn deserialize(_data: &[u8]) -> Option<Self> {
        // Dummy deserialization
        None
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

        let _msg = SwarmMessage::IntentBroadcast {
            intent_id,
            description: String::from(description),
            confidence,
        };

        // In a real implementation, we would loop over all peers in NODE and send.
        // For the prototype, we simulate the broadcast logic.
        println!(
            "Swarm [{}]: Broadcasting intent '{}' (ID: {})",
            self.node_id, description, intent_id
        );

        self.active_intents
            .lock()
            .insert(intent_id, String::from(description));
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
            SwarmMessage::Heartbeat { _capabilities: _ } => {
                // println!("Swarm [{}]: Heartbeat from Node {} with {} capabilities", self.node_id, sender_id, capabilities.len());
            }
        }
    }
}

lazy_static! {
    pub static ref AGENT: Arc<SwarmAgent> = Arc::new(SwarmAgent::new(1));
}

/// Background task to handle swarm coordination and message processing.
pub async fn swarm_task() {
    println!("Swarm: Agent initialized.");

    loop {
        // 1. Check for incoming MeshPackets from the networking layer
        // 2. Decrypt and deserialize into SwarmMessages
        // 3. Call AGENT.handle_message()

        // Simulate a periodic heartbeat broadcast
        // let msg = SwarmMessage::Heartbeat { capabilities: alloc::vec![String::from("audio"), String::from("computation")] };
        // let data = msg.serialize();
        // NODE.send_to(target_peer, &data);

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
