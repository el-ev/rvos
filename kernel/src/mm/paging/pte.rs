use core::fmt;

use bitflags::bitflags;

use crate::mm::{
    addr::{PhysAddr, PhysPageNum},
    consts::{PTEFLAGS_MASK, PTE_PPN_MASK},
    frame,
};

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PteFlags: u16 {
        /// Valid
        const V = 1 << 0;
        /// Readable
        const R = 1 << 1;
        /// Writable
        const W = 1 << 2;
        /// Executable
        const X = 1 << 3;
        /// User mode accessible
        const U = 1 << 4;
        /// Global
        const G = 1 << 5;
        /// Accessed
        const A = 1 << 6;
        /// Dirty
        const D = 1 << 7;
        // Copy on write
        const RSW1 = 1 << 8;
        const COW = 1 << 8;
        // Reserved for software
        const RSW2 = 1 << 9;
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    bits: usize,
}

impl PageTableEntry {
    pub const fn new(ppn: PhysPageNum, flags: PteFlags) -> Self {
        Self {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }

    pub const EMPTY: Self = Self { bits: 0 };

    pub fn clear(&mut self) {
        self.bits = 0;
    }

    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum((self.bits & PTE_PPN_MASK) << 2)
    }

    pub fn pa(&self) -> PhysAddr {
        self.ppn().into()
    }

    pub fn flags(&self) -> PteFlags {
        PteFlags::from_bits_truncate((self.bits & PTEFLAGS_MASK) as u16)
    }

    pub fn valid(&self) -> bool {
        self.flags().contains(PteFlags::V)
    }

    pub fn readable(&self) -> bool {
        self.flags().contains(PteFlags::R)
    }

    pub fn writable(&self) -> bool {
        self.flags().contains(PteFlags::W)
    }

    pub fn executable(&self) -> bool {
        self.flags().contains(PteFlags::X)
    }

    pub fn user(&self) -> bool {
        self.flags().contains(PteFlags::U)
    }

    pub fn accessed(&self) -> bool {
        self.flags().contains(PteFlags::A)
    }

    pub fn dirty(&self) -> bool {
        self.flags().contains(PteFlags::D)
    }

    pub fn global(&self) -> bool {
        self.flags().contains(PteFlags::G)
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "PTE @ {:p}", self)?;
        writeln!(f, "  bits: {:#018x}", self.bits)?;
        writeln!(f, "  ppn: {}", self.ppn())?;
        writeln!(f, "  flags: {:?}", self.flags())
    }
}
