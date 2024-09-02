// QEMU virt machine 10MHz
pub const CLOCK_FREQ: usize = 10_000_000;

pub const INTERRUPT_PER_SEC: usize = 100;

pub const MACHINE_TICKS_PER_USEC: usize = CLOCK_FREQ / USEC_PER_SEC;
pub const USEC_PER_INTERRUPT: usize = USEC_PER_SEC / INTERRUPT_PER_SEC;

pub const MSEC_PER_SEC: usize = 1_000;
pub const USEC_PER_SEC: usize = 1_000_000;
pub const NSEC_PER_SEC: usize = 1_000_000_000;
