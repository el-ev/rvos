use log::warn;

pub use phys::{PhysAddr, PhysPageNum};
pub use virt::{VirtAddr, VirtPageNum};

use super::{address_space::{KERNEL_OFFSET, K_PHYSICAL_MEMORY_BEG, K_VIRTUAL_MEMORY_BEG, PHYSICAL_MEMORY_START}, consts::PAGE_SIZE};
use crate::config::MEMORY_SIZE;

mod phys;
mod virt;

pub fn pa2kva(pa: PhysAddr) -> VirtAddr {
    if !(PHYSICAL_MEMORY_START..PHYSICAL_MEMORY_START + MEMORY_SIZE).contains(&pa.0) {
        warn!("Address not in physical memory range");
    }
    VirtAddr(pa.0 + KERNEL_OFFSET)
}

pub fn kva2pa(va: VirtAddr) -> PhysAddr {
    if !(K_PHYSICAL_MEMORY_BEG..K_PHYSICAL_MEMORY_BEG + MEMORY_SIZE).contains(&va.0) {
        warn!("Address not in kernel virtual memory range");
    }
    PhysAddr(va.0 - KERNEL_OFFSET)
}

impl PhysAddr {
    pub unsafe fn as_slice(self, len: usize) -> &'static [u8] {
        let mapped_addr = pa2kva(self);
        core::slice::from_raw_parts(mapped_addr.0 as *const u8, len)
    }

    pub unsafe fn as_mut_slice(self, len: usize) -> &'static mut [u8] {
        let mapped_addr = pa2kva(self);
        core::slice::from_raw_parts_mut(mapped_addr.0 as *mut u8, len)
    }

    pub unsafe fn as_page_slice(self) -> &'static [u8] {
        self.as_slice(PAGE_SIZE)
    }

    pub unsafe fn as_mut_page_slice(self) -> &'static mut [u8] {
        self.as_mut_slice(PAGE_SIZE)
    }
}
