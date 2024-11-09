pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}
/**
 * bump allocator is a simple allocator that
 * allocates memory by bumping a pointer
 * and is deallocated all at once
 * it always allocates memory in a contiguous block
 * the next pointer is incremented by the size of the allocation
 * and the deallocate function resets the next pointer
 */

impl BumpAllocator {
    /// Creates a new empty bump allocator.
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;
use super::{align_up, Locked};

// heap allocator need to implement the GlobalAlloc trait
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {

        let mut allocator = self.lock(); // get a mutable reference to the allocator
        let alloc_start = align_up(allocator.next, layout.align());
        // checked_add returns None if the operation overflows
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };
        if alloc_end > allocator.heap_end {
            ptr::null_mut() // out of memory
        } else {
            allocator.next = alloc_end;
            allocator.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut allocator = self.lock();

        allocator.allocations -= 1;
        if allocator.allocations == 0 {
            allocator.next = allocator.heap_start;
        }
    }
}