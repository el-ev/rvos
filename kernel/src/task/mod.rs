#![allow(unused)]

use core::sync::atomic::AtomicBool;

use alloc::{task, vec};
use taskdef::TaskControlBlock;

use crate::include_bytes_align_as;

pub mod hart;
pub mod pid;
pub mod schedule;
pub mod taskdef;
pub mod user_space;

const LOOP: &[u8] = include_bytes_align_as!(usize, "../../../target/riscv64gc-unknown-none-elf/debug/dummy");
const PAGEFAULT: &[u8] = include_bytes_align_as!(usize, "../../../user/pagefault.b");

pub static TASK_PREPARED: AtomicBool = AtomicBool::new(false); 

pub fn run() -> ! {
    let task = TaskControlBlock::new();
    task.clone().init(LOOP, vec![]);
    schedule::SCHEDULER.new_task(task);
    // let task = TaskControlBlock::new();
    // task.clone().init(PAGEFAULT, vec![]);
    // let _ = schedule::SCHEDULER.add_task(task);
    TASK_PREPARED.store(true, core::sync::atomic::Ordering::SeqCst);
    schedule::SCHEDULER.main_loop()
}
