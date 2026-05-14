use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    sync::atomic::{AtomicU64, Ordering},
};
use alloc::boxed::Box;

pub mod executor;
pub mod keyboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    High = 0,
    Normal = 1,
    Low = 2,
}

pub struct Task {
    pub id: TaskId,
    pub(crate) future: Pin<Box<dyn Future<Output = ()>>>,
    pub priority: Priority,
    pub ticks_remaining: usize,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
            priority: Priority::Normal,
            ticks_remaining: 10, // Default quantum
        }
    }

    pub fn with_priority(future: impl Future<Output = ()> + 'static, priority: Priority) -> Task {
        let quantum = match priority {
            Priority::High => 5,
            Priority::Normal => 10,
            Priority::Low => 20,
        };
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
            priority,
            ticks_remaining: quantum,
        }
    }

    pub(crate) fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
