// Define the ListNode struct, which represents a node in a linked list of free memory blocks.
// Each node points to the next free block of memory (if available).
struct ListNode {
    next: Option<&'static mut ListNode>, // 'next' stores the address of the next free block.
}

// Define the block sizes to use, which will determine the allocator's granularity.
// These sizes must be powers of two, as they are used for alignment, and alignments must
// also be powers of two for proper memory management.
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

// The FixedSizeBlockAllocator structure manages memory in fixed-size blocks.
// It uses multiple linked lists to store free blocks of various sizes, as defined in BLOCK_SIZES.
// For memory that doesn't fit these sizes, it uses a fallback allocator.
pub struct FixedSizeBlockAllocator {
    // Array of linked list heads for each block size, storing the available free blocks.
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    // Fallback allocator for cases when a specific block size is unavailable.
    fallback_allocator: linked_list_allocator::Heap, 
    // This allocator doesn't merge adjacent free blocks, but it can still manage memory 
    // outside the fixed-size blocks.
}

use alloc::alloc::{GlobalAlloc, Layout};
use core::{ptr::{self, NonNull}, mem};

impl FixedSizeBlockAllocator {
    /// Creates an empty FixedSizeBlockAllocator with no initialized blocks.
    /// Sets all the linked list heads to `None`, meaning no blocks are currently free.
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()], // Initialize the free lists as empty
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initialize the allocator with a specific heap memory region.
    /// 
    /// This function is `unsafe` because the caller must guarantee that the specified
    /// memory region is valid, unused, and exclusive to the allocator.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size); // Initialize fallback allocator
    }
    
    /// Uses the fallback allocator to allocate memory when no suitable fixed-size block is available.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        // Try to allocate memory using the fallback allocator and return a pointer to the allocated memory.
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(), // Successful allocation returns the memory pointer
            Err(_) => ptr::null_mut(), // Allocation failure returns a null pointer
        }
    }
}

/// Helper function to select the appropriate block size for a given layout.
/// Returns an index in `BLOCK_SIZES` array that represents the smallest block
/// size that can fit the requested layout.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align()); // Ensure alignment is considered
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size) // Find smallest suitable block size
}

use super::Locked;

// Implement the GlobalAlloc trait, which allows the allocator to be used as a global allocator.
unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock(); // Lock the allocator for thread-safe access

        // Determine the appropriate list index for the requested allocation size and alignment.
        match list_index(&layout) {
            Some(index) => {
                // Try to take a free block from the list head of the appropriate size.
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        // If a free block is available, set the head of the list to the next node.
                        allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8 // Return the address of the allocated block
                    }
                    None => {
                        // No block of the required size is available; allocate a new block.
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout) // Use fallback allocator
                    }
                }
            }
            None => allocator.fallback_alloc(layout), // Fallback for unsupported block sizes
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock(); // Lock the allocator for thread-safe access

        // Determine the appropriate list index for the block being deallocated.
        match list_index(&layout) {
            Some(index) => {
                // Create a new ListNode to represent the freed block.
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };

                // Validate the block's size and alignment before adding it back to the list.
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

                // Write the new node to the memory location being freed.
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);

                // Set the list head for this block size to the newly freed node.
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => {
                // For blocks not matching our fixed sizes, use the fallback allocator's deallocation.
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout)
            }
        }
    }
}
