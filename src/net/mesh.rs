use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::ChaCha20Poly1305;
use core::sync::atomic::{AtomicU64, Ordering};
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use spinning_top::Spinlock;
use x25519_dalek::{PublicKey, StaticSecret};

pub const NONCE_SIZE: usize = 12;

#[derive(Debug, Clone)]
pub enum MeshError {
    PeerNotFound,
    EncryptionFailed,
    DecryptionFailed,
}

pub type Result<T> = core::result::Result<T, MeshError>;

/// Represents a secure packet in the mesh network.
#[derive(Debug, Clone)]
pub struct MeshPacket {
    pub sender_id: u64,
    pub nonce: [u8; NONCE_SIZE],
    pub payload: Vec<u8>,
}

/// A node in the secure mesh network.
pub struct MeshNode {
    node_id: u64,
    secret: StaticSecret,
    public_key: PublicKey,
    peers: Spinlock<BTreeMap<u64, MeshPeer>>,
}

/// Metadata and session state for a remote peer.
pub struct MeshPeer {
    pub public_key: PublicKey,
    pub cipher: ChaCha20Poly1305,
}

impl MeshNode {
    /// Creates a new mesh node with a cryptographically secure identity.
    pub fn new(node_id: u64) -> Self {
        let mut seed = [0u8; 32];
        crate::entropy::fill_entropy(&mut seed);

        let mut rng = ChaCha20Rng::from_seed(seed);
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);

        let secret = StaticSecret::from(secret_bytes);
        let public_key = PublicKey::from(&secret);

        Self {
            node_id,
            secret,
            public_key,
            peers: Spinlock::new(BTreeMap::new()),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    /// Adds a peer to the mesh and establishes a shared secret via ECDH.
    pub fn add_peer(&self, peer_id: u64, peer_public_key: PublicKey) {
        let shared_secret = self.secret.diffie_hellman(&peer_public_key);
        let cipher = ChaCha20Poly1305::new(shared_secret.as_bytes().into());

        self.peers.lock().insert(
            peer_id,
            MeshPeer {
                public_key: peer_public_key,
                cipher,
            },
        );
        // Note: We removed the println here to avoid deadlock risks in Wakers/ISRs.
    }

    /// Encrypts a payload for a specific peer.
    pub fn send_to(&self, peer_id: u64, payload: &[u8]) -> Result<MeshPacket> {
        let mut peers = self.peers.lock();
        let peer = peers.get_mut(&peer_id).ok_or(MeshError::PeerNotFound)?;

        // Nonces must NEVER be reused for the same key.
        // We use a global atomic counter to ensure uniqueness within a boot cycle.
        // In a production system with persistent storage, this must be persisted.
        static NONCE_COUNTER: AtomicU64 = AtomicU64::new(1);
        let seq = NONCE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[0..8].copy_from_slice(&seq.to_le_bytes());

        let encrypted = peer
            .cipher
            .encrypt(&nonce.into(), payload)
            .map_err(|_| MeshError::EncryptionFailed)?;

        Ok(MeshPacket {
            sender_id: self.node_id,
            nonce,
            payload: encrypted,
        })
    }

    /// Decrypts an incoming packet from a specific peer.
    pub fn receive_from(&self, packet: MeshPacket) -> Result<Vec<u8>> {
        let mut peers = self.peers.lock();
        let peer = peers
            .get_mut(&packet.sender_id)
            .ok_or(MeshError::PeerNotFound)?;

        peer.cipher
            .decrypt(&packet.nonce.into(), packet.payload.as_ref())
            .map_err(|_| MeshError::DecryptionFailed)
    }

    /// Returns a list of all active peer IDs in the mesh.
    pub fn get_peer_ids(&self) -> Vec<u64> {
        self.peers.lock().keys().cloned().collect()
    }

    /// Receives the next available packet from the network.
    pub fn receive_next(&self) -> Option<MeshPacket> {
        // In a real system, this would drain from a NIC buffer or a task queue.
        None
    }
}

/// Verifies the cryptographic handshake and encryption cycle.
#[cfg(feature = "test")]
pub fn test_mesh_crypto() {
    crate::serial_print!("test_mesh_crypto... ");

    let node_a = MeshNode::new(10);
    let node_b = MeshNode::new(20);

    // 1. Exchange keys via out-of-band discovery
    node_a.add_peer(20, node_b.public_key());
    node_b.add_peer(10, node_a.public_key());

    // 2. Send encrypted message from A to B
    let message = b"Hello from Node A";
    let packet = node_a
        .send_to(20, message)
        .expect("Failed to create packet");

    // 3. Decrypt at B
    let decrypted = node_b.receive_from(packet).expect("Failed to decrypt");

    assert_eq!(message, decrypted.as_slice());
    crate::serial_println!("[ok]");
}

lazy_static::lazy_static! {
    pub static ref NODE: Arc<MeshNode> = Arc::new(MeshNode::new(1));
}

/// Background task to handle mesh coordination.
pub async fn mesh_task() {
    // Initial log can stay as it's called during startup, not from ISR
    crate::println!("Mesh: Node initialized. ID: 1");

    loop {
        // 1. Scan for other nodes via mDNS (to be implemented)
        // 2. Perform handshakes
        // 3. Exchange system state securely

        // Use a real yield to let other tasks run efficiently
        crate::task::yield_now().await;
    }
}
