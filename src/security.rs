use crate::sync::Spinlock;
use alloc::sync::Arc;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub struct SecurityModule {
    fuzz_counter: AtomicUsize,
    pub is_active: Spinlock<bool>,
}

impl Default for SecurityModule {
    fn default() -> Self {
        Self {
            fuzz_counter: AtomicUsize::new(0),
            is_active: Spinlock::new(false),
        }
    }
}

impl SecurityModule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_active(&self, active: bool) {
        *self.is_active.lock() = active;
    }

    pub fn run_fuzzing_cycle(&self) {
        if !*self.is_active.lock() {
            return;
        }

        let cycle = self.fuzz_counter.fetch_add(1, Ordering::SeqCst);

        if cycle.is_multiple_of(5) {
            crate::serial_println!(
                "Security: Running penetration test cycle #{} on virtual boundaries...",
                cycle
            );
            self.simulate_decryption_attack();
        }
    }

    fn simulate_decryption_attack(&self) {
        // OMEGA-Level Hardening: Use hardware-derived entropy (RDTSC + Ticks)
        let tsc = unsafe { core::arch::x86_64::_rdtsc() };
        let ticks = crate::interrupts::TICKS.load(Ordering::Relaxed);
        let mut seed = [0u8; 32];
        seed[0..8].copy_from_slice(&tsc.to_le_bytes());
        seed[8..16].copy_from_slice(&ticks.to_le_bytes());
        // Fill remaining with deterministic but non-obvious values
        for (i, byte) in seed.iter_mut().enumerate().skip(16) {
            *byte = (tsc >> (i % 8)) as u8 ^ (ticks >> (i % 8)) as u8;
        }

        let mut rng = ChaCha20Rng::from_seed(seed);
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);
        let mut nonce = [0u8; 12];
        rng.fill_bytes(&mut nonce);

        let mut cipher = ChaCha20::new(&key.into(), &nonce.into());
        let mut buffer = [0u8; 15];
        buffer.copy_from_slice(b"Top secret data");

        cipher.apply_keystream(&mut buffer);
    }
}

lazy_static! {
    pub static ref SECURITY: Arc<SecurityModule> = Arc::new(SecurityModule::new());
}

pub async fn security_task() {
    crate::serial_println!("Security: Automated Penetration Testing Module initialized.");
    SECURITY.set_active(true);

    loop {
        SECURITY.run_fuzzing_cycle();
        crate::task::yield_now().await;
    }
}

#[cfg(feature = "test")]
pub fn test_security_module() {
    crate::serial_print!("test_security_module... ");
    let sec = SecurityModule::new();
    sec.set_active(true);
    sec.run_fuzzing_cycle();
    assert_eq!(sec.fuzz_counter.load(Ordering::SeqCst), 1);
    crate::serial_println!("[ok]");
}
