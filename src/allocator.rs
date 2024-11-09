use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError, FrameAllocator,
    },
    VirtAddr,
};

pub mod bump;

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Align the address `addr` upwards to alignment `align`.
fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // addr already aligned
    } else {
        addr - remainder + align
    }
    // better implementation where align is a power of 2
    // (addr + (align - 1)) & !(align - 1)
}

pub struct Dummy;  

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should never be called")
    }
}

use bump::BumpAllocator;

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

pub const HEAP_SIZE: usize = 1024 * 1024; // 1 MB
pub const HEAP_START: usize = 0x4444_4444_0000;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,             
    // `mapper` is responsible for mapping virtual pages to physical frames.
    frame_allocator: &mut impl FrameAllocator<Size4KiB>, 
    // `frame_allocator` is responsible for allocating physical frames for pages.
) -> Result<(), MapToError<Size4KiB>> { // Returns `Ok(())` on success, or a `MapToError` if there is an error.
    
    // Define the range of pages that will represent the heap.
    let page_range = {
        // Start of the heap in virtual memory.
        let heap_start = VirtAddr::new(HEAP_START as u64);        
        // End of the heap in virtual memory, calculated by adding the heap size.
        let heap_end = heap_start + HEAP_SIZE - 1u64;        
        // Find the first page that contains the start of the heap.
        let heap_start_page = Page::containing_address(heap_start);        
        // Find the last page that contains the end of the heap.
        let heap_end_page = Page::containing_address(heap_end);        
        // Create a range of pages from the start page to the end page, inclusive.
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // For each page in the calculated range, allocate a frame and map it to that page.
    for page in page_range {
        // Allocate a physical frame for this page. If no frame is available, return an error.
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;        
        // Map the virtual `page` to the allocated physical `frame` with the specified flags.
        // The `unsafe` keyword is required because `map_to` may involve modifying memory directly.
        unsafe {
            let _ = mapper.map_to(page, frame, flags, frame_allocator)?;
        };
    }

    // Initialize the linked list allocator with the start and size of the heap.
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    // Return success if all pages were successfully mapped.
    Ok(())
}
