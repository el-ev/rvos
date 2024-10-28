use crate::{
    error::OsError,
    mm::{address_space::U_HEAP_BEG, consts::PAGE_SIZE},
};
use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use bitflags::bitflags;
use log::trace;

use crate::{
    config::TASK_STACK_SIZE,
    mm::{
        addr::{VirtAddr, VirtPageNum},
        address_space::U_STACK_END,
        frame::{self, FrameTracker},
        paging::{page_table::PageTable, pte::PteFlags},
    },
};

pub struct UserSpace {
    pub page_table: PageTable,
    areas: BTreeMap<VirtPageNum, UserArea>,
}

impl UserSpace {
    pub fn new() -> Self {
        Self {
            page_table: PageTable::from_kernel_page_table(),
            areas: BTreeMap::new(),
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
            let mut vpn = start.floor_page();
            for i in (0..size).step_by(PAGE_SIZE) {
                let mut area = UserArea::new(UserAreaType::Framed, perm, vpn);
                area.map(&mut self.page_table)
                    .expect("failed to map user area");
                area.copy_data(
                    &mut self.page_table,
                    &elf.input[offset + i..(offset + i + PAGE_SIZE).min(offset + size)],
                );
                self.areas.insert(vpn, area);
                vpn += 1;
            }
        }
        // alloc stack
        trace!("allocating stack");
        (U_STACK_END - TASK_STACK_SIZE..U_STACK_END)
            .rev()
            .step_by(PAGE_SIZE)
            .for_each(|va| {
                let vpn = VirtAddr(va).floor_page();
                let area =
                    UserArea::new(UserAreaType::Framed, UserAreaPerm::R | UserAreaPerm::W, vpn);
                self.areas.insert(vpn, area);
            });
        elf.header.pt2.entry_point() as usize
    }

    pub fn init_heap(&mut self, page_count: usize) {
        let heap_start = VirtAddr(U_HEAP_BEG);
        let mut vpn = heap_start.floor_page();
        for _ in 0..page_count {
            let area = UserArea::new(UserAreaType::Framed, UserAreaPerm::R | UserAreaPerm::W, vpn);
            // area.map(&mut self.page_table)
            //     .expect("failed to map user area");
            self.areas.insert(vpn, area);
            vpn += 1;
        }
    }

    pub fn check_perm(&self, vpn: VirtPageNum, perm: UserAreaPerm) -> bool {
        if let Some(area) = self.areas.get(&vpn) {
            area.perm().contains(perm)
        } else {
            false
        }
    }

    pub fn alloc(&mut self, vpn: VirtPageNum, perm: UserAreaPerm) -> Result<(), OsError> {
        if self.areas.contains_key(&vpn) {
            return Ok(());
        }
        let area = UserArea::new(UserAreaType::Framed, perm, vpn);
        // area.map(&mut self.page_table)?;
        self.areas.insert(vpn, area);
        Ok(())
    }

    pub fn handle_page_fault(&mut self, stval: usize, ty: UserPageFaultType) -> Result<(), ()> {
        let vpn = VirtAddr(stval).floor_page();
        let perm = match ty {
            UserPageFaultType::Read => UserAreaPerm::R,
            UserPageFaultType::Write => UserAreaPerm::R | UserAreaPerm::W,
            UserPageFaultType::Execute => UserAreaPerm::R | UserAreaPerm::X,
        };
        if !self.check_perm(vpn, perm) {
            return Err(());
        }
        self.areas
            .get_mut(&vpn)
            .unwrap()
            .map(&mut self.page_table)
            .map_err(|_| ())?;
        Ok(())
    }

    pub fn find_frame(&mut self, vpn: VirtPageNum) -> Result<Arc<FrameTracker>, OsError> {
        if let Some(area) = self.areas.get_mut(&vpn) {
            if !area.is_mapped() {
                area.map(&mut self.page_table)?;
            }
            Ok(area.get_frame())
        } else {
            Err(OsError::InvalidParam)
        }
    }

    pub fn map(&mut self, vpn: VirtPageNum, frame: Arc<FrameTracker>, perm: UserAreaPerm) {
        let mut area = UserArea::new_with_frame(UserAreaType::Framed, perm, vpn, frame);
        area.map(&mut self.page_table).unwrap();
        self.areas.insert(vpn, area);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) -> Result<(), OsError> {
        if let Some(area) = self.areas.get_mut(&vpn) {
            area.unmap(&mut self.page_table);
            self.areas.remove(&vpn);
            Ok(())
        } else {
            Err(OsError::InvalidParam)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UserPageFaultType {
    Read,
    Write,
    Execute,
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
    _ty: UserAreaType,
    perm: UserAreaPerm,
    frame: Option<Arc<FrameTracker>>,
    vpn: VirtPageNum,
}

impl UserArea {
    fn new(ty: UserAreaType, perm: UserAreaPerm, vpn: VirtPageNum) -> Self {
        Self {
            _ty: ty,
            perm,
            frame: None,
            vpn,
        }
    }

    fn new_with_frame(ty: UserAreaType, perm: UserAreaPerm, vpn: VirtPageNum, frame: Arc<FrameTracker>) -> Self {
        Self {
            _ty: ty,
            perm,
            frame: Some(frame),
            vpn,
        }
    }

    fn perm(&self) -> UserAreaPerm {
        self.perm
    }

    fn map(&mut self, page_table: &mut PageTable) -> Result<(), OsError> {
        trace!("mapping user area: {:x?}, perm: {:?}", self.vpn, self.perm);
        if self.frame.is_none() {
            let frame = frame::alloc()?;
            self.frame = Some(Arc::new(frame));
        }
        // TODO When type is not framed (file mapping)
        page_table.map(
            self.vpn,
            self.frame.as_ref().unwrap().ppn,
            self.perm.as_pte_flag(),
        );
        Ok(())
    }

    fn unmap(&mut self, page_table: &mut PageTable) {
        if self.frame.is_some() {
            page_table.unmap(self.vpn);
            self.frame = None;
        }
    }

    fn copy_data(&self, page_table: &mut PageTable, data: &[u8]) {
        let dst = unsafe { page_table.query(self.vpn).unwrap().pa().as_mut_page_slice() };
        dst[..data.len()].copy_from_slice(data);
    }

    fn is_mapped(&self) -> bool {
        self.frame.is_some()
    }

    fn get_frame(&self) -> Arc<FrameTracker> {
        debug_assert!(self.frame.is_some());
        self.frame.clone().unwrap()
    }
}
