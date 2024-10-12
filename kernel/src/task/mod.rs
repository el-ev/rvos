#![allow(unused)]

use alloc::vec;
use taskdef::TaskControlBlock;

pub mod pid;
pub mod taskdef;
mod user_space;

const LOOP: &[u8] = include_bytes!("../../../user/loop.b");
pub fn run() {
    let task = TaskControlBlock::new();
    task.init(LOOP, vec![]);
    
}