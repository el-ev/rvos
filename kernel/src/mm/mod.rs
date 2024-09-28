#![allow(unused)]

use addr::{kva2pa, PhysAddr, VirtAddr};
use address_space::PHYSICAL_MEMORY_START;

use crate::config::MEMORY_SIZE;

pub mod addr;
pub mod address_space;
pub mod consts;
pub mod frame;
mod heap;
pub mod layout;
pub mod paging;

unsafe extern "C" {
    fn __kernel_end();
}

pub fn init() {
    heap::init();
    heap::heap_test();
    frame::init(
        kva2pa(VirtAddr(__kernel_end as usize)),
        PhysAddr(PHYSICAL_MEMORY_START + MEMORY_SIZE),
    );
    layout::print_memory_layout();
    // frame::debug_print();
}
