#![allow(unused)]

use core::sync::atomic::AtomicBool;

use alloc::{task, vec};
use arch::tp;
use log::info;
use sbi::legacy::sbi_send_ipi;
use taskdef::TaskControlBlock;

use crate::{get_hart_count, include_bytes_align_as, mask, mm::address_space::K_HARDWARE_BEG};

pub mod hart;
pub mod pid;
pub mod schedule;
pub mod taskdef;
pub mod user_space;

const LOOP: &[u8] = include_bytes_align_as!(
    usize,
    "../../../target/riscv64gc-unknown-none-elf/debug/dummy"
);
const PAGEFAULT: &[u8] = include_bytes_align_as!(usize, "../../../user/pagefault.b");

pub fn run() -> ! {
    let task = TaskControlBlock::new();
    task.clone().init(LOOP, vec![]);
    schedule::SCHEDULER.new_task(task);
    let task = TaskControlBlock::new();
    task.clone().init(LOOP, vec![]);
    schedule::SCHEDULER.new_task(task);
    let task = TaskControlBlock::new();
    task.clone().init(LOOP, vec![]);
    schedule::SCHEDULER.new_task(task);
    // let task = TaskControlBlock::new();
    // task.clone().init(PAGEFAULT, vec![]);
    // let _ = schedule::SCHEDULER.add_task(task);

    sbi_send_ipi(mask!(get_hart_count()) & !(1 << tp()));
    schedule::SCHEDULER.main_loop()
}
