// TODO: Auto modify CPU_NUM when makefile changes
pub const CPU_NUM: usize = 8;

pub const KERNEL_HEAP_SIZE: usize = 0x100_0000; // 16MiB

pub const MEMORY_SIZE: usize = 0x8000_0000; // 2GiB

pub const PHYSICAL_MEMORY_START: usize = 0x8000_0000;

pub const KERNEL_VIRTUAL_MEMORY_START: usize = 0xFFFF_FFFF_0000_0000;

pub const KERNEL_OFFSET: usize = KERNEL_VIRTUAL_MEMORY_START - PHYSICAL_MEMORY_START;

// TODO Add device memory region
