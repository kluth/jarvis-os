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
        let mut hash = [0u8; 32];
        for (i, &byte) in data.iter().enumerate() {
            hash[i % 32] ^= byte;
        }
        Self(hash)
    }
}

pub struct DhtNode {
    node_id: u64,
    fragments: Spinlock<BTreeMap<ChunkHash, Vec<u8>>>,
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

    pub fn store_file(&self, data: &[u8]) -> usize {
        let file_id = self.file_counter.fetch_add(1, Ordering::SeqCst);
        let mut hashes = Vec::new();
        let mut fragments = self.fragments.lock();

        for chunk in data.chunks(CHUNK_SIZE) {
            let hash = ChunkHash::new(chunk);
            hashes.push(hash.clone());
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

    pub fn retrieve_file(&self, file_id: usize) -> Option<Vec<u8>> {
        let hashes = {
            let index = self.file_index.lock();
            index.get(&file_id)?.clone()
        };

        let mut data = Vec::new();
        let fragments = self.fragments.lock();

        for hash in hashes {
            let chunk = fragments.get(&hash)?;
            data.extend_from_slice(chunk);
        }

        Some(data)
    }
}

lazy_static! {
    pub static ref DHT: Arc<DhtNode> = Arc::new(DhtNode::new(1));
}

pub async fn dht_task() {
    println!("Storage: Decentralized Hash Table node initialized.");
    loop {
        crate::task::yield_now().await;
    }
}

#[cfg(feature = "test")]
pub fn test_dht_storage() {
    crate::serial_print!("test_dht_storage... ");
    let node = DhtNode::new(99);
    let test_data = alloc::vec![0x42; 100];
    let file_id = node.store_file(&test_data);
    let retrieved = node
        .retrieve_file(file_id)
        .expect("Failed to retrieve file");
    assert_eq!(test_data.len(), retrieved.len());
    assert_eq!(test_data[0..10], retrieved[0..10]);
    crate::serial_println!("[ok]");
}
