use arch::tp;
use sbi::legacy::sbi_send_ipi;
use taskdef::TaskControlBlock;

use crate::{get_hart_count, include_bytes_align_as, mask};

pub mod hart;
pub mod pid;
pub mod schedule;
pub mod taskdef;
pub mod user_space;

const LOOP: &[u8] = include_bytes_align_as!(
    usize,
    "../../../target/riscv64gc-unknown-none-elf/debug/dummy"
);
// const PAGEFAULT: &[u8] = include_bytes_align_as!(usize, "../../../user/pagefault.b");

pub fn run() -> ! {
    let task = TaskControlBlock::new();
    task.clone().init(LOOP);
    task.set_priority(2);
    let _ = schedule::SCHEDULER.submit_task(task);
    for _ in 0..25 {
        let task = TaskControlBlock::new();
        task.clone().init(LOOP);
        schedule::SCHEDULER
            .submit_task(task)
            .expect("submit task failed");
    }

    sbi_send_ipi(mask!(get_hart_count()) & !(1 << tp()));
    schedule::SCHEDULER.hart_loop()
}
