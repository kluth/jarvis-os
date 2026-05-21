use crate::sync::Spinlock;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        Self { size, next: None }
    }
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
    used_bytes: usize,
    total_size: usize,
}

impl Default for LinkedListAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
            used_bytes: 0,
            total_size: 0,
        }
    }
    /// Initializes the allocator with a block of memory.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory block starting at `heap_start` is valid
    /// and large enough to hold `heap_size` bytes.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
        self.total_size = heap_size;
    }
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        let align = core::mem::align_of::<ListNode>();
        assert_eq!(align_up(addr, align), addr);
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr);
    }
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head as *mut ListNode;
        loop {
            let next_ptr = unsafe {
                let next_ref = (*current).next.as_mut()?;
                *next_ref as *mut ListNode
            };
            let region = unsafe { &mut *next_ptr };
            if let Ok(alloc_start) = alloc_from_region(region, size, align) {
                let next = region.next.take();
                let ret = unsafe { (*current).next.take().unwrap() };
                unsafe { (*current).next = next };
                return Some((ret, alloc_start));
            }
            current = next_ptr;
        }
    }
    pub fn used(&self) -> usize {
        self.used_bytes
    }
    pub fn size(&self) -> usize {
        self.total_size
    }
}

fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
    let alloc_start = align_up(region.start_addr(), align);
    let alloc_end = alloc_start.checked_add(size).ok_or(())?;
    if alloc_end > region.end_addr() {
        return Err(());
    }
    let excess_size = region.end_addr() - alloc_end;
    if excess_size > 0 && excess_size < core::mem::size_of::<ListNode>() {
        return Err(());
    }
    Ok(alloc_start)
}

fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr
    } else {
        addr - remainder + align
    }
}

pub struct LockedAllocator {
    inner: Spinlock<LinkedListAllocator>,
}

impl Default for LockedAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl LockedAllocator {
    pub const fn new() -> Self {
        Self {
            inner: Spinlock::new(LinkedListAllocator::new()),
        }
    }
    pub fn lock(&self) -> crate::sync::SpinlockGuard<'_, LinkedListAllocator> {
        self.inner.lock()
    }
}

unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.inner.lock();
        let size = align_up(
            core::cmp::max(layout.size(), core::mem::size_of::<ListNode>()),
            core::mem::align_of::<ListNode>(),
        );
        if let Some((region, alloc_start)) = allocator.find_region(size, layout.align()) {
            let alloc_end = alloc_start + size;
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            allocator.used_bytes += size;
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.inner.lock();
        let size = align_up(
            core::cmp::max(layout.size(), core::mem::size_of::<ListNode>()),
            core::mem::align_of::<ListNode>(),
        );
        allocator.add_free_region(ptr as usize, size);
        allocator.used_bytes -= size;
    }
}
