#![allow(unused)]

use core::ptr::addr_of;

use addr::{PhysAddr, PhysPageNum, VirtAddr, kva2pa};
use address_space::{K_HARDWARE_BEG, K_HARDWARE_END, PHYSICAL_MEMORY_START};
use paging::page_table::PageTable;
use paging::pte::{PageTableEntry, PteFlags};

use crate::config::MEMORY_SIZE;
use crate::entry::BOOT_PAGE_TABLE;

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

pub fn map_device_region() {
    // let mut pt = unsafe {
    //     PageTable::from_ppn(
    //         kva2pa(VirtAddr(addr_of!(BOOT_PAGE_TABLE) as usize)).into()
    //     )
    // };
    unsafe {BOOT_PAGE_TABLE[511] = PageTableEntry::new(PhysAddr(0x0).into(), PteFlags::R | PteFlags::W | PteFlags::V)};
    
    // for i in (K_HARDWARE_BEG..K_HARDWARE_END).step_by(4096) {
    //     pt.map(
    //         VirtAddr(i).into(),
    //         PhysAddr(i - K_HARDWARE_BEG).into(),
    //         PteFlags::R | PteFlags::W,
    //     );
    // }
}
