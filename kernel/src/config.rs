// TODO: Auto modify CPU_NUM when makefile changes
pub const CPU_NUM: usize = 4;

pub const KERNEL_HEAP_SIZE: usize = 0x100_0000; // 16MiB

pub const MEMORY_SIZE: usize = 0x8000_0000; // 2GiB

pub const TASK_STACK_SIZE: usize = 0x8000; // 32KiB

pub const MAX_TASKS: usize = 1024;