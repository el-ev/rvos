#![allow(unused)]

use alloc::vec;
use taskdef::TaskControlBlock;

use crate::include_bytes_align_as;

pub mod pid;
pub mod schedule;
pub mod taskdef;
mod user_space;

const LOOP: &[u8] = include_bytes_align_as!(usize, "../../../user/loop.b");

pub fn run() -> ! {
    let task = TaskControlBlock::new();
    task.init(LOOP, vec![]);
    schedule::SCHEDULER.main_loop()
}
