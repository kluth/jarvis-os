use core::{
    mem,
    ptr::{self, NonNull},
};
use super::Locked;

/// A simple slab allocator for fixed-size objects.
pub struct Slab<T> {
    size: usize,
    free_list: Option<NonNull<SlabEntry>>,
    _phantom: core::marker::PhantomData<T>,
}

struct SlabEntry {
    next: Option<NonNull<SlabEntry>>,
}

impl<T> Slab<T> {
    pub const fn new() -> Self {
        assert!(mem::size_of::<T>() >= mem::size_of::<SlabEntry>());
        Slab {
            size: mem::size_of::<T>(),
            free_list: None,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Initializes the slab with a block of memory.
    pub unsafe fn init(&mut self, start_addr: usize, count: usize) {
        for i in 0..count {
            let ptr = (start_addr + i * self.size) as *mut SlabEntry;
            self.free_node(NonNull::new_unchecked(ptr));
        }
    }

    pub fn allocate(&mut self) -> Option<*mut T> {
        self.free_list.map(|node| {
            unsafe {
                self.free_list = node.as_ref().next;
                node.as_ptr() as *mut T
            }
        })
    }

    pub unsafe fn deallocate(&mut self, ptr: *mut T) {
        self.free_node(NonNull::new_unchecked(ptr as *mut SlabEntry));
    }

    unsafe fn free_node(&mut self, mut node: NonNull<SlabEntry>) {
        node.as_mut().next = self.free_list;
        self.free_list = Some(node);
    }
}

unsafe impl<T> Send for Slab<T> {}

pub struct SlabAllocator<T> {
    inner: Locked<Slab<T>>,
}

impl<T> SlabAllocator<T> {
    pub const fn new() -> Self {
        SlabAllocator {
            inner: Locked::new(Slab::new()),
        }
    }

    pub unsafe fn init(&self, start_addr: usize, count: usize) {
        self.inner.lock().init(start_addr, count);
    }

    pub fn alloc(&self) -> Option<*mut T> {
        self.inner.lock().allocate()
    }

    pub unsafe fn dealloc(&self, ptr: *mut T) {
        self.inner.lock().deallocate(ptr);
    }
}
