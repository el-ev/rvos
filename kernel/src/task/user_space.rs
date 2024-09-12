use bitflags::bitflags;

use crate::{config::TASK_STACK_SIZE, mm::{addr::{VirtAddr, VirtPageNum}, address_space::U_STACK_END, paging::{page_table::PageTable, pte::PteFlags}}};

pub struct UserSpace {
    pub page_table: PageTable
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackId(usize);

impl StackId {
    pub fn stack_bottom(&self) -> VirtAddr {
        VirtAddr(U_STACK_END - self.0 * TASK_STACK_SIZE)
    }

    
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtAddrRange {
    start: VirtAddr,
    end: VirtAddr,
}

impl VirtAddrRange {
    pub fn new(start: VirtAddr, end: VirtAddr) -> Self {
        assert!(start <= end);
        VirtAddrRange { start, end }
    }

    pub fn new_from_size(start: VirtAddr, size: usize) -> Self {
        VirtAddrRange::new(start, VirtAddr(start.0 + size))
    }

    pub fn size(&self) -> usize {
        self.end.0 - self.start.0
    }

    pub fn start(&self) -> VirtAddr {
        self.start
    }

    pub fn end(&self) -> VirtAddr {
        self.end
    }

    pub fn contains(&self, addr: VirtAddr) -> bool {
        self.start <= addr && addr < self.end
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn iter(&self) -> VARangeVPNIter {
        VARangeVPNIter {
            range: *self,
            curr: self.start.floor_page(),
        }
    }
}

pub struct VARangeVPNIter {
    range: VirtAddrRange,
    curr: VirtPageNum,
}

bitflags! {
    pub struct UserAreaPerm: usize {
        const R = 1 << 0;
        const W = 1 << 1;
        const X = 1 << 2;
    }
}

impl UserAreaPerm {
    pub fn to_pte_flag(&self) -> PteFlags {
        let mut pte_flag = PteFlags::V | PteFlags::U;
        if self.contains(UserAreaPerm::R) {
            pte_flag |= PteFlags::R;
        }
        if self.contains(UserAreaPerm::W) {
            pte_flag |= PteFlags::W;
        }
        if self.contains(UserAreaPerm::X) {
            pte_flag |= PteFlags::X;
        }
        pte_flag
    }
}

pub enum UserAreaType {
    Normal,
    File, // TODO
}

pub struct UserArea {

}