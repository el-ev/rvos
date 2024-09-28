use core::ptr::addr_of_mut;

use allocator::BuddyAllocator;
use log::info;

use crate::config::KERNEL_HEAP_SIZE;

#[global_allocator]
static HEAP_ALLOCATOR: BuddyAllocator<32> = BuddyAllocator::new();
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .add_size(addr_of_mut!(HEAP_SPACE) as usize, KERNEL_HEAP_SIZE);
    }
    info!(
        "Initialized {} KiB of kernel heap.",
        KERNEL_HEAP_SIZE / 1024
    );
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

pub fn heap_test() {
    let mut vec = alloc::vec![0; 1000];
    vec.iter_mut()
        .enumerate()
        .take(1000)
        .for_each(|(i, x)| *x = i);
    vec.iter()
        .enumerate()
        .take(1000)
        .for_each(|(i, x)| assert_eq!(i, *x));
}
