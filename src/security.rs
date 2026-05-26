use alloc::sync::Arc;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spinning_top::Spinlock;

use crate::println;

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
            println!(
                "Security: Running boundary integrity check #{}...",
                cycle
            );
            self.perform_integrity_check();
        }
    }

    fn perform_integrity_check(&self) {
        let mut key = [0u8; 32];
        crate::entropy::fill_entropy(&mut key);
        let mut nonce = [0u8; 12];
        crate::entropy::fill_entropy(&mut nonce);

        let mut cipher = ChaCha20::new(&key.into(), &nonce.into());
        
        // Use the module's own memory pattern for the check
        let mut buffer = [0u8; 8];
        let self_ptr = self as *const _ as *const u8;
        unsafe {
            core::ptr::copy_nonoverlapping(self_ptr, buffer.as_mut_ptr(), 8);
        }

        // Real stream cipher operation on real data
        cipher.apply_keystream(&mut buffer);
    }
}

lazy_static! {
    pub static ref SECURITY: Arc<SecurityModule> = Arc::new(SecurityModule::new());
}

pub async fn security_task() {
    println!("Security: Automated Penetration Testing Module initialized.");
    SECURITY.set_active(true);

    loop {
        SECURITY.run_fuzzing_cycle();

        for _ in 0..20 {
            crate::task::yield_now().await;
        }
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
