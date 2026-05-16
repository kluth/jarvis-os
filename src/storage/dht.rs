use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spinning_top::Spinlock;

use crate::println;

const CHUNK_SIZE: usize = 4096; // 4KB chunks

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkHash([u8; 32]);

impl ChunkHash {
    pub fn new(data: &[u8]) -> Self {
        // Simple hash for the prototype. In production, use SHA-256 or similar.
        let mut hash = [0u8; 32];
        for (i, &byte) in data.iter().enumerate() {
            hash[i % 32] ^= byte;
        }
        Self(hash)
    }
}

pub struct DhtNode {
    node_id: u64,
    /// Local storage of fragments
    fragments: Spinlock<BTreeMap<ChunkHash, Vec<u8>>>,
    /// File index: maps file ID to a list of fragment hashes
    file_index: Spinlock<BTreeMap<usize, Vec<ChunkHash>>>,
    file_counter: AtomicUsize,
}

impl DhtNode {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            fragments: Spinlock::new(BTreeMap::new()),
            file_index: Spinlock::new(BTreeMap::new()),
            file_counter: AtomicUsize::new(1),
        }
    }

    /// Stores a file by fragmenting it, hashing the chunks, and distributing them.
    pub fn store_file(&self, data: &[u8]) -> usize {
        let file_id = self.file_counter.fetch_add(1, Ordering::SeqCst);
        let mut hashes = Vec::new();
        let mut fragments = self.fragments.lock();

        for chunk in data.chunks(CHUNK_SIZE) {
            let hash = ChunkHash::new(chunk);
            hashes.push(hash.clone());

            // In a real DHT, we'd calculate distance to peers and route the chunk.
            // For the prototype, we store it locally to represent 'our' shards.
            fragments.insert(hash, chunk.to_vec());
        }

        self.file_index.lock().insert(file_id, hashes);
        println!(
            "DHT [Node {}]: Stored file {} ({} chunks)",
            self.node_id,
            file_id,
            data.len() / CHUNK_SIZE + 1
        );

        file_id
    }

    /// Reconstructs a file from fragments over the mesh.
    pub fn retrieve_file(&self, file_id: usize) -> Option<Vec<u8>> {
        let hashes = {
            let index = self.file_index.lock();
            index.get(&file_id)?.clone()
        };

        let mut data = Vec::new();
        let fragments = self.fragments.lock();

        for hash in hashes {
            // Here we would normally query the DHT network for the chunk.
            if let Some(chunk) = fragments.get(&hash) {
                data.extend_from_slice(chunk);
            } else {
                println!(
                    "DHT [Node {}]: Failed to retrieve chunk for file {}",
                    self.node_id, file_id
                );
                return None;
            }
        }

        Some(data)
    }
}

lazy_static! {
    pub static ref DHT: Arc<DhtNode> = Arc::new(DhtNode::new(1));
}

/// Background task to handle DHT peer queries.
pub async fn dht_task() {
    println!("Storage: Decentralized Hash Table node initialized.");

    loop {
        // Handle incoming requests for chunks or store requests from peers
        crate::task::yield_now().await;
    }
}

#[cfg(feature = "test")]
pub fn test_dht_storage() {
    crate::serial_print!("test_dht_storage... ");
    let node = DhtNode::new(99);

    // Create a 10KB test file
    let test_data = alloc::vec![0x42; 10240];

    // Store it
    let file_id = node.store_file(&test_data);

    // Retrieve it
    let retrieved = node
        .retrieve_file(file_id)
        .expect("Failed to retrieve file");

    assert_eq!(test_data.len(), retrieved.len());
    assert_eq!(test_data[0..10], retrieved[0..10]);

    crate::serial_println!("[ok]");
}
