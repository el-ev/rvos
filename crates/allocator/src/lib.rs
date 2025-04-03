#![no_std]
extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ops::Deref;
use sync::SpinNoIrqMutex;

mod buddy;
mod list;

use buddy::Heap;

pub struct LockedAllocator<const ORDER: usize> {
    heap: SpinNoIrqMutex<Heap<ORDER>>,
}

impl<const ORDER: usize> Default for LockedAllocator<ORDER> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const ORDER: usize> LockedAllocator<ORDER> {
    pub const fn new() -> Self {
        LockedAllocator {
            heap: SpinNoIrqMutex::new(Heap::new()),
        }
    }
}

impl<const ORDER: usize> Deref for LockedAllocator<ORDER> {
    type Target = SpinNoIrqMutex<Heap<ORDER>>;

    fn deref(&self) -> &Self::Target {
        &self.heap
    }
}

unsafe impl<const ORDER: usize> GlobalAlloc for LockedAllocator<ORDER> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.heap
            .lock()
            .alloc(layout)
            .map_or(core::ptr::null_mut(), |ptr| ptr.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.heap
            .lock()
            .dealloc(unsafe { core::ptr::NonNull::new_unchecked(ptr) }, layout)
    }
}
