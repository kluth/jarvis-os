use crate::sync::Spinlock;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll, Waker};

/// A thread-safe signal for event-driven task execution.
pub struct Signal {
    triggered: AtomicBool,
    waker: Spinlock<Option<Waker>>,
}

impl Signal {
    pub const fn new() -> Self {
        Self {
            triggered: AtomicBool::new(false),
            waker: Spinlock::new(None),
        }
    }
}

impl Default for Signal {
    fn default() -> Self {
        Self::new()
    }
}

impl Signal {
    /// Triggers the signal and wakes the awaiting task.
    pub fn trigger(&self) {
        self.triggered.store(true, Ordering::SeqCst);
        if let Some(waker) = self.waker.lock().take() {
            waker.wake();
        }
    }

    /// Resets the signal state.
    pub fn reset(&self) {
        self.triggered.store(false, Ordering::SeqCst);
    }

    /// Returns a future that resolves when the signal is triggered.
    pub fn wait(&self) -> SignalWait<'_> {
        SignalWait { signal: self }
    }
}

pub struct SignalWait<'a> {
    signal: &'a Signal,
}

impl<'a> Future for SignalWait<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.signal.triggered.load(Ordering::SeqCst) {
            Poll::Ready(())
        } else {
            *self.signal.waker.lock() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
