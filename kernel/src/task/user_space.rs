use crate::{
    Mutex, entry,
    mm::{addr::PhysPageNum, address_space::U_HEAP_BEG, consts::PAGE_SIZE},
};
use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use bitflags::bitflags;

use crate::{
    config::TASK_STACK_SIZE,
    mm::{
        addr::{VPNRange, VirtAddr, VirtPageNum},
        address_space::U_STACK_END,
        frame::{self, FrameTracker},
        paging::{
            page_table::{self, PageTable},
            pte::PteFlags,
        },
    },
    utils::pool::UsizePool,
};

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

    pub fn map_elf(&mut self, elf: &[u8]) -> usize {
        let elf = xmas_elf::ElfFile::new(elf).expect("failed to parse ELF file");
        for ph in elf.program_iter() {
            if ph.get_type().expect("failed to get program header type")
                != xmas_elf::program::Type::Load
            {
                continue;
            }
            let offset = ph.offset() as usize;
            let start = VirtAddr(ph.virtual_addr() as usize);
            let size = ph.mem_size() as usize;
            let perm = ph.flags().into();
            let mut area = UserArea::new(UserAreaType::Framed, perm, start, start + size);
            area.map(&mut self.page_table);
            area.copy_data(&mut self.page_table, &elf.input[offset..offset + size]);
            self.areas.push(area);
        }

        // alloc stack
        let stack_bottom = U_STACK_END - TASK_STACK_SIZE;
        let stack_top = U_STACK_END;
        let mut stack_area = UserArea::new(
            UserAreaType::Framed,
            UserAreaPerm::R | UserAreaPerm::W,
            stack_bottom.into(),
            stack_top.into(),
        );
        stack_area.map(&mut self.page_table);
        self.areas.push(stack_area);
        elf.header.pt2.entry_point() as usize
    }

    pub fn init_stack(&mut self, args: Vec<String>) -> usize {
        // TODO
        0
    }

    pub fn init_heap(&mut self, page_count: usize) {
        let size = page_count * PAGE_SIZE;
        let heap_start = U_HEAP_BEG;
        let heap_end = heap_start + size;
        let mut heap_area = UserArea::new(
            UserAreaType::Framed,
            UserAreaPerm::R | UserAreaPerm::W,
            heap_start.into(),
            heap_end.into(),
        );
        heap_area.map(&mut self.page_table);
        self.areas.push(heap_area);
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

impl From<xmas_elf::program::Flags> for UserAreaPerm {
    fn from(flags: xmas_elf::program::Flags) -> Self {
        let mut perm = UserAreaPerm::empty();
        if flags.is_read() {
            perm |= UserAreaPerm::R;
        }
        if flags.is_write() {
            perm |= UserAreaPerm::W;
        }
        if flags.is_execute() {
            perm |= UserAreaPerm::X;
        }
        perm
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
        let ppn = match self.ty {
            UserAreaType::Framed => {
                let frame = frame::alloc().expect("failed to allocate frame for user area");
                let ppn = frame.ppn;
                self.frames.insert(vpn, frame);
                ppn
            }
        };
        page_table.map(vpn, ppn, self.perm.as_pte_flag());
    }

    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.range().iter() {
            self.unmap_one(vpn, page_table);
        }
    }

    pub fn unmap_one(&mut self, vpn: VirtPageNum, page_table: &mut PageTable) {
        if self.ty == UserAreaType::Framed {
            self.frames.remove(&vpn).expect("frame not found");
        }
        page_table.unmap(vpn);
    }

    pub fn copy_data(&self, page_table: &mut PageTable, data: &[u8]) {
        let mut current = 0;
        let mut iter = self.range.iter();
        while current < data.len() {
            let vpn = iter.next().expect("data too large");
            let src = &data[current..data.len().min(current + PAGE_SIZE)];
            let dst = unsafe {page_table.query(vpn).unwrap().pa().as_mut_page_slice()};
            dst[..src.len()].copy_from_slice(src);
            current += src.len();
            iter.next();
        }
    }
}
