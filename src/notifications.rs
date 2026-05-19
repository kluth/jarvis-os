use crate::sync::Spinlock;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextState {
    Active,
    DoNotDisturb,
    Sleep,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: usize,
    pub message: String,
    pub priority: Priority,
    pub timestamp: u64,
}

pub struct NotificationCenter {
    queue: Spinlock<Vec<Notification>>,
    counter: AtomicUsize,
    pub state: Spinlock<ContextState>,
}

impl Default for NotificationCenter {
    fn default() -> Self {
        Self {
            queue: Spinlock::new(Vec::new()),
            counter: AtomicUsize::new(0),
            state: Spinlock::new(ContextState::Active),
        }
    }
}

impl NotificationCenter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_state(&self, state: ContextState) {
        *self.state.lock() = state;
    }

    pub fn push(&self, message: &str, priority: Priority) {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let state = self.state.lock().clone();

        // Filtering logic based on context
        let should_push = match state {
            ContextState::Active => true,
            ContextState::DoNotDisturb => {
                priority == Priority::High || priority == Priority::Critical
            }
            ContextState::Sleep => priority == Priority::Critical,
        };

        if should_push {
            self.queue.lock().push(Notification {
                id,
                message: String::from(message),
                priority,
                timestamp: 0, // Placeholder
            });
        }
    }

    pub fn pop(&self) -> Option<Notification> {
        let mut q = self.queue.lock();
        if q.is_empty() {
            None
        } else {
            Some(q.remove(0)) // Pop front
        }
    }
}

lazy_static! {
    pub static ref CENTER: Arc<NotificationCenter> = Arc::new(NotificationCenter::new());
}

#[cfg(feature = "test")]
pub fn test_notifications() {
    crate::serial_print!("test_notifications... ");
    let center = NotificationCenter::new();

    center.set_state(ContextState::DoNotDisturb);
    center.push("Low priority", Priority::Low); // Blocked
    center.push("High priority", Priority::High); // Allowed

    assert_eq!(center.queue.lock().len(), 1);

    let n = center.pop().unwrap();
    assert_eq!(n.message, "High priority");

    crate::serial_println!("[ok]");
}
