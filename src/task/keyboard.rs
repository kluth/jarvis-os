use crate::drivers::keyboard;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Some(queue) = SCANCODE_QUEUE.get() {
        if queue.push(scancode).is_err() {
            // Drop scancode if queue is full
        } else {
            WAKER.wake();
        }
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl Default for ScancodeStream {
    fn default() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl ScancodeStream {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.get().expect("not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());
        if let Some(scancode) = queue.pop() {
            WAKER.take();
            Poll::Ready(Some(scancode))
        } else {
            Poll::Pending
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();

    while let Some(scancode) = scancodes.next().await {
        if let Some(character) = keyboard::scancode_to_char(scancode) {
            crate::serial_print!("{}", character);
        }
    }
}
