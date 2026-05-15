use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

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

/// A future that yields execution back to the executor.
pub struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn yield_now() -> YieldNow {
    YieldNow { yielded: false }
}
