#![no_std]
use core::arch::asm;

use riscv::register::sstatus;

#[inline(always)]
pub fn fp() -> usize {
    let mut value: usize;
    unsafe {
        asm!("mv {}, fp", out(reg) value);
    }
    value
}

#[inline(always)]
pub fn ra() -> usize {
    let mut value: usize;
    unsafe {
        asm!("mv {}, ra", out(reg) value);
    }
    value
}

#[inline(always)]
pub fn sp() -> usize {
    let mut value: usize;
    unsafe {
        asm!("mv {}, sp", out(reg) value);
    }
    value
}

#[inline(always)]
pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}

#[inline]
pub fn get_hart_id() -> usize {
    let hart_id;
    unsafe { core::arch::asm!("mv {0}, tp", out(reg) hart_id) };
    hart_id
}

#[inline]
pub fn enable_sie() {
    unsafe { sstatus::set_sie() };
}

#[inline]
pub fn disable_sie() {
    unsafe { sstatus::clear_sie() };
}

#[inline]
pub fn read_sie() -> bool {
    sstatus::read().sie()
}