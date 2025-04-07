use core::alloc::GlobalAlloc;

use crate::consts::PAGE_SIZE;

mod bitmap;
mod buddy;
mod list;

pub struct Allocator {
    heap: sync::SpinMutex<buddy::Heap<20>>,
}

impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator {
    pub const fn new() -> Self {
        Allocator {
            heap: sync::SpinMutex::new(buddy::Heap::new()),
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.heap
            .lock()
            .alloc(layout)
            .map_or(core::ptr::null_mut(), |ptr| ptr.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.heap
            .lock()
            .dealloc(unsafe { core::ptr::NonNull::new_unchecked(ptr) }, layout)
    }
}
