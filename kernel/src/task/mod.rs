#![allow(unused)]

use core::sync::atomic::AtomicBool;

use alloc::{task, vec};
use taskdef::TaskControlBlock;

use crate::{get_hart_count, include_bytes_align_as};

pub mod hart;
pub mod pid;
pub mod schedule;
pub mod taskdef;
pub mod user_space;

pub static TASK_PREPARED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

const LOOP: &[u8] = include_bytes_align_as!(usize, "../../../target/riscv64gc-unknown-none-elf/debug/dummy");
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
    TASK_PREPARED.store(true, core::sync::atomic::Ordering::SeqCst);
    schedule::SCHEDULER.main_loop()
}
