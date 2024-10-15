use core::mem::MaybeUninit;

use alloc::sync::Arc;
use arch::tp;
use sync::Lazy;

use crate::{config::CPU_NUM, Mutex};

use super::taskdef::TaskControlBlock;

pub struct HartLocal {
    pub hart_id: usize,
    pub current_task: Option<Arc<TaskControlBlock>>,
}

static mut HART_LOCAL: Lazy<[Mutex<HartLocal>; CPU_NUM]> = Lazy::new(|| {
    let mut array = [const { MaybeUninit::uninit() }; CPU_NUM];
    for i in 0..CPU_NUM {
        array[i] = MaybeUninit::new(Mutex::new(HartLocal {
            hart_id: i,
            current_task: None,
        }));
    }
    unsafe { core::mem::transmute::<_, [Mutex<HartLocal>; CPU_NUM]>(array) }
});

#[inline(always)]
pub fn get_hart_id() -> usize {
    tp()
}

pub fn get_current_task() -> Option<Arc<TaskControlBlock>> {
    let hart_id = get_hart_id();
    unsafe { &HART_LOCAL[hart_id] }.lock().current_task.clone()
}

pub fn set_current_task(task: Arc<TaskControlBlock>) {
    let hart_id = get_hart_id();
    unsafe { &HART_LOCAL[hart_id] }.lock().current_task = Some(task);
}