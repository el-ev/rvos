use alloc::sync::Arc;

use crate::task::taskdef::TaskControlBlock;

#[repr(usize)]
enum Syscall {
    A=0,
}

pub fn do_syscall(task: Arc<TaskControlBlock>) {
    
}