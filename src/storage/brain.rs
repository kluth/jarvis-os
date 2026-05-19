use crate::storage::merkle;
use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrainTier {
    Local,         // Immediate working memory (RAM)
    Central,       // Shared aid/coordination
    Decentralized, // Immutable hive knowledge (DHT)
}

#[derive(Debug, Clone)]
pub struct Synapse {
    pub key: String,
    pub value: Vec<u8>,
    pub tier: BrainTier,
    pub importance: u8, // 0-255 (Semantic importance for biological pruning)
    pub last_access: u64,
}

pub struct JarvisBrain {
    pub synapses: Spinlock<BTreeMap<String, Synapse>>,
    pub hive_root: AtomicUsize, // Root hash of decentralized knowledge
    pub synaptic_density: AtomicUsize,
}

lazy_static! {
    pub static ref CORE: JarvisBrain = JarvisBrain {
        synapses: Spinlock::new(BTreeMap::new()),
        hive_root: AtomicUsize::new(0),
        synaptic_density: AtomicUsize::new(0),
    };
}

impl JarvisBrain {
    pub fn store(&self, key: &str, value: Vec<u8>, tier: BrainTier, importance: u8) {
        let mut synapses = self.synapses.lock();

        // Biological redundancy check: If it's decentralized, generate Merkle Root
        if tier == BrainTier::Decentralized {
            let blocks = alloc::vec![value.clone()];
            let tree = merkle::MerkleTree::from_data_blocks(&blocks);
            self.hive_root
                .store(tree.root_hash as usize, Ordering::SeqCst);
            crate::serial_println!("[BRAIN] Immutability locked: CID for '{}' registered.", key);
        }

        synapses.insert(
            String::from(key),
            Synapse {
                key: String::from(key),
                value,
                tier,
                importance,
                last_access: 0, // Placeholder
            },
        );

        self.synaptic_density
            .store(synapses.len(), Ordering::SeqCst);
    }

    pub fn retrieve(&self, key: &str) -> Option<Vec<u8>> {
        let synapses = self.synapses.lock();
        synapses.get(key).map(|s| s.value.clone())
    }

    /// Synaptic Pruning: Biological cache eviction
    /// Removes low-importance "synapses" when memory is tight.
    pub fn prune(&self, threshold: u8) {
        let mut synapses = self.synapses.lock();
        let initial_count = synapses.len();

        synapses.retain(|_, s| {
            // Keep decentralized knowledge always, prune local low-importance
            s.tier == BrainTier::Decentralized || s.importance >= threshold
        });

        let pruned = initial_count - synapses.len();
        if pruned > 0 {
            crate::serial_println!(
                "[BRAIN] Synaptic pruning complete: {} low-weight memories evicted.",
                pruned
            );
            self.synaptic_density
                .store(synapses.len(), Ordering::SeqCst);
        }
    }
}

pub async fn brain_task() {
    crate::serial_println!("[BRAIN] Tiered Intelligence Engine initialized.");

    // Simulate biological growth
    CORE.store(
        "CORE_DIRECTIVE",
        alloc::vec![1, 2, 3],
        BrainTier::Decentralized,
        255,
    );
    CORE.store(
        "TEMPORARY_CONTEXT",
        alloc::vec![0; 1024],
        BrainTier::Local,
        10,
    );

    loop {
        // Periodic biological maintenance
        // If density > 50, prune anything with importance < 20
        if CORE.synaptic_density.load(Ordering::SeqCst) > 50 {
            CORE.prune(20);
        }

        // Simulate Central Aid coordination
        let density = CORE.synaptic_density.load(Ordering::SeqCst);
        if density > 0 && density.wrapping_rem(5) == 0 {
            // crate::serial_println!("[BRAIN] Syncing with Central Intelligence Hub...");
        }

        for _ in 0..100 {
            crate::task::yield_now().await;
        }
    }
}
