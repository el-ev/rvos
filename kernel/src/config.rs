pub const CPU_NUM: usize = 4;

pub const KERNEL_HEAP_SIZE: usize = 0x100_0000; // 16MiB

pub const TASK_STACK_SIZE: usize = 0x8000; // 32KiB

pub const MAX_TASKS: usize = 1024;

// -- From device tree

pub static mut MEMORY_SIZE: usize = 0;

pub static mut UART_BASE: usize = 0;

pub static mut PLIC_BASE: usize = 0;
// TODO Add device memory region

