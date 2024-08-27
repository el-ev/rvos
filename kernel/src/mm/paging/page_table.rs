use alloc::vec;
use alloc::vec::Vec;

use super::pte::{PageTableEntry, PteFlags};

use crate::mask;
use crate::mm::addr::{pa2kva, PhysAddr, PhysPageNum, VirtPageNum};
use crate::mm::consts::PPN_WIDTH;
use crate::mm::frame::{self, FrameTracker};
use crate::mm::{addr::VirtAddr, consts::PAGE_TABLE_ENTRY_COUNT as ENTRY_COUNT};

impl PhysPageNum {
    fn as_page_table(&self) -> &'static mut [PageTableEntry] {
        let va = pa2kva((*self).into());
        unsafe { core::slice::from_raw_parts_mut(va.0 as *mut PageTableEntry, ENTRY_COUNT) }
    }
}

impl VirtPageNum {
    fn indices(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut indices = [0; 3];
        for i in (0..3).rev() {
            indices[i] = vpn & mask!(9);
            vpn >>= 9;
        }
        indices
    }
}

#[derive(Debug)]
pub struct PageTable {
    ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame::alloc().expect("failed to allocate frame for page table");
        Self {
            ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    pub fn from_satp(satp: usize) -> Self {
        let ppn = PhysPageNum(satp & mask!(PPN_WIDTH));
        Self {
            ppn,
            frames: vec![],
        }
    }

    pub fn find(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indices = vpn.indices();
        let mut page_table = self.ppn;
        let mut result = None;
        for (i, index) in indices.iter().enumerate() {
            let pte = &mut page_table.as_page_table()[*index];
            if !pte.valid() {
                return None;
            }
            if i == 2 {
                result = Some(pte);
                return result;
            }
            page_table = pte.ppn();
        }
        result
    }

    pub fn find_create(&mut self, vpn: VirtPageNum) -> &mut PageTableEntry {
        let indices = vpn.indices();
        let mut page_table = self.ppn;
        for (i, index) in indices.iter().enumerate() {
            let pte = &mut page_table.as_page_table()[*index];
            if !pte.valid() {
                let frame = frame::alloc().expect("failed to allocate frame for page table");
                *pte = PageTableEntry::new(frame.ppn, PteFlags::V);
                self.frames.push(frame);
            }
            if i == 2 {
                return pte;
            }
            page_table = pte.ppn();
        }
        unreachable!()
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PteFlags) {
        let pte = self.find_create(vpn);
        debug_assert!(!pte.valid());
        *pte = PageTableEntry::new(ppn, flags | PteFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find(vpn).expect("failed to unmap page");
        debug_assert!(pte.valid());
        pte.clear();
    }

    pub fn query(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        Some(*self.find(vpn)?)
    }
}
