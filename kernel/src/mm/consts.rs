use crate::mask;

pub const PAGE_SIZE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SIZE_BITS;

pub const HUGE_PAGE_SIZE_BITS: usize = 30;
pub const HUGE_PAGE_SIZE: usize = 1 << HUGE_PAGE_SIZE_BITS;

pub const FRAME_SIZE: usize = PAGE_SIZE;

// Sv39
pub const VA_WIDTH: usize = 39;
pub const PA_WIDTH: usize = 56;

pub const PPN_WIDTH: usize = PA_WIDTH - PAGE_SIZE_BITS;
pub const VPN_WIDTH: usize = VA_WIDTH - PAGE_SIZE_BITS;

pub const PA_PPN_MASK: usize = mask!(PA_WIDTH) & !mask!(PAGE_SIZE_BITS);

pub const PTEFLAGS_BITS: usize = 10;
pub const PTEFLAGS_MASK: usize = mask!(PTEFLAGS_BITS);

pub const PTE_PPN_MASK: usize = ((1usize << 54) - 1) & !PTEFLAGS_MASK;

pub const PAGE_TABLE_ENTRY_COUNT: usize = 512;
