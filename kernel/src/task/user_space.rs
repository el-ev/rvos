use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use bitflags::bitflags;
use crate::Mutex;

use crate::{config::TASK_STACK_SIZE, mm::{addr::{VPNRange, VirtAddr, VirtPageNum}, address_space::U_STACK_END, frame::{self, FrameTracker}, paging::{page_table::{self, PageTable}, pte::PteFlags}}, utils::pool::UsizePool};

pub struct UserSpace {
    page_table: PageTable,
    areas: Vec<UserArea>,
}

impl UserSpace {
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct UserAreaPerm: usize {
        const R = 1 << 0;
        const W = 1 << 1;
        const X = 1 << 2;
    }
}

impl UserAreaPerm {
    pub fn as_pte_flag(&self) -> PteFlags {
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
    Framed,
}

pub struct UserArea {
    ty: UserAreaType,
    perm: UserAreaPerm,
    frames: BTreeMap<VirtPageNum, FrameTracker>,
    range: VPNRange,
}

impl UserArea {
    pub fn new(ty: UserAreaType, perm: UserAreaPerm, start: VirtAddr, end: VirtAddr) -> Self {
        Self {
            ty,
            perm,
            frames: BTreeMap::new(),
            range: VPNRange::new(start.floor_page(), end.ceil_page()),
        }

    }

    pub fn range(&self) -> VPNRange {
        self.range
    }

    pub fn perm(&self) -> UserAreaPerm {
        self.perm
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.range().iter() {
            self.map_one(vpn, page_table);
        }
    }

    pub fn map_one(&mut self, vpn: VirtPageNum, page_table: &mut PageTable) {
        let frame = frame::alloc().expect("failed to allocate frame for user area");
        let ppn = frame.ppn;
        self.frames.insert(vpn, frame);
        page_table.map(vpn, ppn, self.perm.as_pte_flag());
    }

    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.range().iter() {
            self.unmap_one(vpn, page_table);
        }
    }

    pub fn unmap_one(&mut self, vpn: VirtPageNum, page_table: &mut PageTable) {
        let frame = self.frames.remove(&vpn).expect("frame not found");
        page_table.unmap(vpn);
        // TODO: frame dropped
    }
}

static STACK_ID_POOL: Mutex<UsizePool> = Mutex::new(UsizePool::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackId(usize);

impl StackId {
    pub fn stack_bottom(&self) -> VirtAddr {
        VirtAddr(U_STACK_END - self.0 * TASK_STACK_SIZE)
    }
}

pub fn alloc_stack_id() -> StackId {
    StackId(STACK_ID_POOL.lock().alloc())
}