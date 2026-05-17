use alloc::vec::Vec;
use core::hash::Hasher;

/// A simple mock hash for no_std demonstration.
/// In production, this would be BLAKE3 or SHA-256.
pub type Hash32 = u32;

pub fn calculate_hash(data: &[u8]) -> Hash32 {
    let mut hasher = fnv::FnvHasher::default();
    hasher.write(data);
    hasher.finish() as u32
}

mod fnv {
    use core::hash::Hasher;

    pub struct FnvHasher(u64);

    impl Default for FnvHasher {
        fn default() -> Self {
            FnvHasher(0xcbf29ce484222325)
        }
    }

    impl Hasher for FnvHasher {
        fn finish(&self) -> u64 {
            self.0
        }

        fn write(&mut self, bytes: &[u8]) {
            for &byte in bytes {
                self.0 ^= byte as u64;
                self.0 = self.0.wrapping_mul(0x100000001b3);
            }
        }
    }
}

pub struct MerkleTree {
    pub root_hash: Hash32,
}

impl MerkleTree {
    pub fn from_data_blocks(blocks: &[Vec<u8>]) -> Self {
        if blocks.is_empty() {
            return Self { root_hash: 0 };
        }

        let mut current_level: Vec<Hash32> = blocks.iter().map(|b| calculate_hash(b)).collect();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    let mut combined = [0u8; 8];
                    combined[0..4].copy_from_slice(&chunk[0].to_le_bytes());
                    combined[4..8].copy_from_slice(&chunk[1].to_le_bytes());
                    next_level.push(calculate_hash(&combined));
                } else {
                    next_level.push(chunk[0]); // Promote odd one up
                }
            }
            current_level = next_level;
        }

        Self {
            root_hash: current_level[0],
        }
    }

    pub fn verify(root_hash: Hash32, data_block: &[u8], _index: usize) -> bool {
        // Simplified verification for the prototype:
        // In a real system, we'd provide a Merkle Proof (path).
        // Here we just show the concept.
        let block_hash = calculate_hash(data_block);
        // This is a placeholder for path-based verification
        block_hash != 0 && root_hash != 0
    }
}
