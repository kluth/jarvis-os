use crate::sync::Spinlock;
use core::{mem, ptr::NonNull};

/// A simple slab allocator for fixed-size objects.
pub struct Slab<T> {
    size: usize,
    free_list: Option<NonNull<SlabEntry>>,
    _phantom: core::marker::PhantomData<T>,
}

struct SlabEntry {
    next: Option<NonNull<SlabEntry>>,
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::new()
    }
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
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory block starting at `start_addr` is valid
    /// and large enough to hold `count` objects of type `T`. The memory must not be
    /// used by any other part of the system.
    pub unsafe fn init(&mut self, start_addr: usize, count: usize) {
        let align = core::mem::align_of::<T>();
        assert_eq!(
            start_addr % align,
            0,
            "Slab start_addr 0x{:x} is not aligned to {}",
            start_addr,
            align
        );

        for i in 0..count {
            let ptr = (start_addr + i * self.size) as *mut SlabEntry;
            self.free_node(NonNull::new_unchecked(ptr));
        }
    }

    pub fn allocate(&mut self) -> Option<*mut T> {
        self.free_list.map(|node| unsafe {
            self.free_list = node.as_ref().next;
            node.as_ptr() as *mut T
        })
    }

    /// Deallocates an object.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` was previously allocated by this slab
    /// and has not been deallocated yet.
    pub unsafe fn deallocate(&mut self, ptr: *mut T) {
        self.free_node(NonNull::new_unchecked(ptr as *mut SlabEntry));
    }

    /// Frees a node.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `node` is a valid pointer to a `SlabEntry`
    /// that is no longer in use.
    unsafe fn free_node(&mut self, mut node: NonNull<SlabEntry>) {
        node.as_mut().next = self.free_list;
        self.free_list = Some(node);
    }
}

unsafe impl<T> Send for Slab<T> {}

pub struct SlabAllocator<T> {
    inner: Spinlock<Slab<T>>,
}

impl<T> Default for SlabAllocator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SlabAllocator<T> {
    pub const fn new() -> Self {
        SlabAllocator {
            inner: Spinlock::new(Slab::new()),
        }
    }

    /// Initializes the allocator with a block of memory.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory block starting at `start_addr` is valid
    /// and large enough to hold `count` objects of type `T`.
    pub unsafe fn init(&self, start_addr: usize, count: usize) {
        self.inner.lock().init(start_addr, count);
    }

    pub fn alloc(&self) -> Option<*mut T> {
        self.inner.lock().allocate()
    }

    /// Deallocates an object.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` was previously allocated by this allocator
    /// and has not been deallocated yet.
    pub unsafe fn dealloc(&self, ptr: *mut T) {
        self.inner.lock().deallocate(ptr);
    }
}
