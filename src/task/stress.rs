use crate::println;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

static STRESS_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub async fn stress_task() {
    println!("Stability: Stress task initialized. Starting allocation pressure...");

    let mut iteration = 0;
    loop {
        iteration += 1;
        STRESS_COUNTER.fetch_add(1, Ordering::SeqCst);

        // 1. Memory Pressure: Allocate and deallocate various sizes
        {
            let mut v = Vec::with_capacity(1024);
            for i in 0..1024 {
                v.push(i as u8);
            }
            // Force a realloc
            v.reserve(2048);
            for i in 0..1024 {
                v.push((i % 256) as u8);
            }
        } // v is dropped here

        // 2. Context Switching: Yield frequently
        if iteration % 100 == 0 {
            crate::task::yield_now().await;
        }

        // 3. Recursive pressure (optional, be careful with stack)
        // We'll just do more allocations for now.
        if iteration % 1000 == 0 {
            println!("Stability: Stress cycle {} completed.", iteration);

            // Large allocation to test heap fragmentation
            let mut large = Vec::with_capacity(1024 * 32);
            for i in 0..1024 * 32 {
                large.push((i % 256) as u8);
            }
        }

        // Small yield to prevent starvation but keep pressure high
        crate::task::yield_now().await;
    }
}
