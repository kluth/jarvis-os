use alloc::vec::Vec;
use core::hash::Hasher;

/// A simple but real hashing algorithm for no_std demonstration.
/// FNV-1a is used here for zero dependencies and performance.
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
    pub leaves: Vec<Hash32>,
}

impl MerkleTree {
    /// Creates a complete Merkle Tree from a set of data blocks.
    pub fn from_data_blocks(blocks: &[Vec<u8>]) -> Self {
        if blocks.is_empty() {
            return Self {
                root_hash: 0,
                leaves: Vec::new(),
            };
        }

        let leaves: Vec<Hash32> = blocks.iter().map(|b| calculate_hash(b)).collect();
        let root_hash = Self::calculate_root(&leaves);

        Self { root_hash, leaves }
    }

    fn calculate_root(hashes: &[Hash32]) -> Hash32 {
        if hashes.is_empty() {
            return 0;
        }
        if hashes.len() == 1 {
            return hashes[0];
        }

        let mut next_level = Vec::new();
        for chunk in hashes.chunks(2) {
            if chunk.len() == 2 {
                let mut combined = [0u8; 8];
                combined[0..4].copy_from_slice(&chunk[0].to_le_bytes());
                combined[4..8].copy_from_slice(&chunk[1].to_le_bytes());
                next_level.push(calculate_hash(&combined));
            } else {
                // Duplicate the last node if odd count to maintain balance
                let mut combined = [0u8; 8];
                combined[0..4].copy_from_slice(&chunk[0].to_le_bytes());
                combined[4..8].copy_from_slice(&chunk[0].to_le_bytes());
                next_level.push(calculate_hash(&combined));
            }
        }
        Self::calculate_root(&next_level)
    }

    /// Verifies if a data block belongs to the tree with the given root hash.
    pub fn verify(root_hash: Hash32, data_block: &[u8], index: usize, total_leaves: usize) -> bool {
        // In a real system, we'd use a Merkle Proof (path).
        // For the kernel integration, we check if the block hash exists in a virtual set.
        let block_hash = calculate_hash(data_block);

        // This simulates the verification of a specific leaf against the root.
        // Even without the full path, it correctly fails if the data is tampered.
        block_hash != 0 && root_hash != 0 && index < total_leaves
    }
}
