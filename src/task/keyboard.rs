use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{task::{Poll, Context}, pin::Pin};
use futures_util::stream::{Stream, StreamExt};
use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
use crate::print;
use futures_util::task::AtomicWaker;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Some(queue) = SCANCODE_QUEUE.get() {
        if let Err(_) = queue.push(scancode) {
            // println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
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
    let mut keyboard = Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    pc_keyboard::DecodedKey::Unicode(character) => print!("{}", character),
                    pc_keyboard::DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
