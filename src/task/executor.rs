use super::{Task, TaskId};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    // MLFQ Queues: 0 = High, 1 = Normal, 2 = Low
    queues: [Arc<ArrayQueue<TaskId>>; 3],
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Default for Executor {
    fn default() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            queues: [
                Arc::new(ArrayQueue::new(100)),
                Arc::new(ArrayQueue::new(100)),
                Arc::new(ArrayQueue::new(100)),
            ],
            waker_cache: BTreeMap::new(),
        }
    }
}

impl Executor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        let priority = task.priority as usize;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.queues[priority].push(task_id).expect("queue full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn run_ready_tasks(&mut self) {
        // Process queues in priority order (MLFQ)
        for i in 0..3 {
            while let Some(task_id) = self.queues[i].pop() {
                let task = match self.tasks.get_mut(&task_id) {
                    Some(task) => task,
                    None => continue,
                };

                let waker = self.waker_cache.entry(task_id).or_insert_with(|| {
                    TaskWaker::from_parts(task_id, self.queues[task.priority as usize].clone())
                });

                let mut context = Context::from_waker(waker);
                match task.poll(&mut context) {
                    Poll::Ready(()) => {
                        self.tasks.remove(&task_id);
                        self.waker_cache.remove(&task_id);
                    }
                    Poll::Pending => {
                        // In a real MLFQ, if a task uses its whole quantum,
                        // it might be demoted to a lower priority queue.
                        // Here, it stays in its queue until next wake.
                    }
                }
            }
        }
    }

    fn sleep_if_idle(&mut self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();
        if self.queues[0].is_empty() && self.queues[1].is_empty() && self.queues[2].is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn from_parts(task_id: TaskId, queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker { task_id, queue }))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self.queue.push(self.task_id);
    }
}
