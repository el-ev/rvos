pub const PAGE_SIZE: usize = 0x1000; // 4KiB
pub const PAGE_SIZE_BITS: usize = 12; // 4KiB

// User space memory layout

pub const DATA_BEG: usize = 0x0000_0000_0001_0000;
pub const DATA_END: usize = 0x0000_0000_1000_0000;

pub const HEAP_BEG: usize = 0x0000_0000_1000_0000;
pub const HEAP_END: usize = 0x0000_0000_2000_0000;
pub const HEAP_SIZE: usize = HEAP_END - HEAP_BEG;

pub const STACK_BEG: usize = 0x0000_0000_2000_0000;
pub const STACK_END: usize = 0x0000_0000_3000_0000;
pub const STACK_SIZE: usize = STACK_END - STACK_BEG;

pub const FILE_MAPPING_BEG: usize = 0x0000_0000_3000_0000;
pub const FILE_MAPPING_END: usize = 0x0000_0000_4000_0000;
pub const FILE_MAPPING_SIZE: usize = FILE_MAPPING_END - FILE_MAPPING_BEG;
