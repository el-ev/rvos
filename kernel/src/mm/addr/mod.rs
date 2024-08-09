use log::warn;

use phys::PhysAddr;
use virt::VirtAddr;

use super::consts::PAGE_SIZE;
use crate::config::{KERNEL_VIRTUAL_MEMORY_START, KERNEL_OFFSET, MEMORY_SIZE, PHYSICAL_MEMORY_START};

mod phys;
mod virt;


pub fn pa2kva(pa: PhysAddr) -> VirtAddr {
    if !(PHYSICAL_MEMORY_START..PHYSICAL_MEMORY_START + MEMORY_SIZE).contains(&pa.0) {
        warn!("Address not in physical memory range");
    }
    VirtAddr(pa.0 + KERNEL_OFFSET)
}

pub fn kva2pa(va: VirtAddr) -> PhysAddr {
    if !(KERNEL_VIRTUAL_MEMORY_START..KERNEL_VIRTUAL_MEMORY_START + MEMORY_SIZE).contains(&va.0) {
        warn!("Address not in kernel virtual memory range");
    }
    PhysAddr(va.0 - KERNEL_OFFSET)
}


impl PhysAddr {
    pub unsafe fn as_slice(&self, len: usize) -> &[u8] {
        let mapped_addr = pa2kva(*self);
        core::slice::from_raw_parts(mapped_addr.0 as *const u8, len)
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn as_mut_slice(&self, len: usize) -> &mut [u8] {
        let mapped_addr = pa2kva(*self);
        core::slice::from_raw_parts_mut(mapped_addr.0 as *mut u8, len)
    }

    pub unsafe fn as_page_slice(&self) -> &[u8] {
        self.as_slice(PAGE_SIZE)
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn as_mut_page_slice(&self) -> &mut [u8] {
        self.as_mut_slice(PAGE_SIZE)
    }
}