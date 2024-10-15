#![allow(unused)]

use alloc::vec;
use taskdef::TaskControlBlock;

use crate::include_bytes_align_as;

pub mod hart;
pub mod pid;
pub mod schedule;
pub mod taskdef;
mod user_space;

const LOOP: &[u8] = include_bytes_align_as!(usize, "../../../target/riscv64gc-unknown-none-elf/debug/dummy");
const PAGEFAULT: &[u8] = include_bytes_align_as!(usize, "../../../user/pagefault.b");

pub fn run() -> ! {
    let task = TaskControlBlock::new();
    task.clone().init(LOOP, vec![]);
    schedule::SCHEDULER.add_task(task);
    // let task = TaskControlBlock::new();
    // task.clone().init(PAGEFAULT, vec![]);
    // schedule::SCHEDULER.add_task(task);
    schedule::SCHEDULER.main_loop()
}
