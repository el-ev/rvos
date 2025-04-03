// User and kernel address space
// Adapted from MankorOS
// See https://gitlab.eduxiji.net/educg-group-18741-1687925/202318123101282-3621/-/blob/final/src/consts/address_space.rs

pub const K_BEG: usize = 0xffff_ffc0_0000_0000;

pub const K_VIRTUAL_MEMORY_BEG: usize = 0xffff_ffc0_0000_0000;
pub const K_VIRTUAL_MEMORY_END: usize = 0xffff_ffd0_0000_0000;

pub const K_FILE_MAPPING_BEG: usize = 0xffff_ffd0_0000_0000;
pub const K_FILE_MAPPING_END: usize = 0xffff_ffe0_0000_0000;

pub const K_PHYSICAL_MEMORY_BEG: usize = 0xffff_fff0_0000_0000;
pub const K_PHYSICAL_MEMORY_END: usize = 0xffff_ffff_8000_0000;

pub const K_HARDWARE_BEG: usize = 0xffff_ffff_8000_0000;
pub const K_HARDWARE_END: usize = 0xffff_ffff_c000_0000;

pub const K_END: usize = 0xffff_ffff_ffff_ffff;

pub const U_BEG: usize = 0x0000_0000_0001_0000;

pub const U_DATA_BEG: usize = 0x0000_0000_0001_0000;
pub const U_DATA_END: usize = 0x0000_0000_1000_0000;

pub const U_HEAP_BEG: usize = 0x0000_0000_1000_0000;
pub const U_HEAP_END: usize = 0x0000_0000_2000_0000;

pub const U_STACK_BEG: usize = 0x0000_0000_2000_0000;
pub const U_STACK_END: usize = 0x0000_0000_3000_0000;

pub const U_FILE_MAPPING_BEG: usize = 0x0000_0000_3000_0000;
pub const U_FILE_MAPPING_END: usize = 0x0000_0000_4000_0000;

pub const U_END: usize = 0x0000_0000_4000_0000;

pub const PHYSICAL_MEMORY_START: usize = 0x8000_0000;
pub const KERNEL_OFFSET: usize = K_PHYSICAL_MEMORY_BEG - PHYSICAL_MEMORY_START;

#[inline]
pub fn is_illegal_user_va(va: usize) -> bool {
    !(U_BEG..U_END).contains(&va)
}

#[inline]
pub fn is_illegal_user_va_range(va: usize, size: usize) -> bool {
    is_illegal_user_va(va) || is_illegal_user_va(va + size - 1)
}
