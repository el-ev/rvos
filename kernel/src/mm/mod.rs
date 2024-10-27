#![allow(unused)]

use core::ptr::addr_of;

use addr::{PhysAddr, PhysPageNum, VirtAddr, kva2pa};
use address_space::{K_HARDWARE_BEG, K_HARDWARE_END, PHYSICAL_MEMORY_START};
use log::debug;
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
        PhysAddr(PHYSICAL_MEMORY_START + unsafe { MEMORY_SIZE }),
    );
    layout::print_memory_layout();
    // frame::debug_print();
}

pub fn map_kernel_regions(dtb: usize) {
    let mut pt =
        unsafe { PageTable::from_ppn(kva2pa(VirtAddr(addr_of!(BOOT_PAGE_TABLE) as usize)).into()) };

    // K_PHYSICAL_MEMORY_BEG - K_PHYSICAL_MEMORY_END (62 GiB)
    // 0xffff_fff0_4000_0000 - 0xffff_ffff_8000_0000
    unsafe {
        for entry in &mut BOOT_PAGE_TABLE[449..511] {
            *entry = PageTableEntry::new(
                PhysPageNum(0x80000),
                PteFlags::R | PteFlags::W | PteFlags::V | PteFlags::X | PteFlags::A | PteFlags::D,
            );
        }
    }
    // K_HARDWARE_BEG - K_HARDWARE_END (1GiB but actually 750 MiB)
    // 0xffff_ffff_8000_0000 - 0xffff_ffff_c000_0000
    unsafe {
        BOOT_PAGE_TABLE[510] = PageTableEntry::new(
            PhysAddr(0x0).into(),
            PteFlags::R | PteFlags::W | PteFlags::V,
        )
    };
}
