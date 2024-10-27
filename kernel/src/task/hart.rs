use core::{mem::MaybeUninit, sync::atomic::AtomicBool};

use alloc::sync::Arc;
use arch::tp;
use sync::Lazy;

use crate::{Mutex, config::CPU_NUM};

use super::taskdef::TaskControlBlock;

pub struct HartLocal {
    #[allow(dead_code)]
    pub hart_id: usize,
    pub ipi_pending: AtomicBool,
    pub current_task: Option<Arc<TaskControlBlock>>,
}

static mut HART_LOCAL: Lazy<[Mutex<HartLocal>; CPU_NUM]> = Lazy::new(|| {
    let mut array = [const { MaybeUninit::uninit() }; CPU_NUM];
    for (i, elem) in array.iter_mut().enumerate() {
        *elem = MaybeUninit::new(Mutex::new(HartLocal {
            hart_id: i,
            ipi_pending: AtomicBool::new(false),
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

pub fn set_current_task(task: Option<Arc<TaskControlBlock>>) {
    let hart_id = get_hart_id();
    unsafe { &HART_LOCAL[hart_id] }.lock().current_task = task;
}

pub fn wake_hart(hart_id: usize) {
    unsafe { &HART_LOCAL[hart_id] }
        .lock()
        .ipi_pending
        .store(true, core::sync::atomic::Ordering::Release);
    sbi::legacy::sbi_send_ipi(1 << hart_id);
}

pub fn clear_ipi() {
    unsafe {
        riscv::register::sip::clear_ssoft();
    }
    let hart_id = get_hart_id();
    unsafe { &HART_LOCAL[hart_id] }
        .lock()
        .ipi_pending
        .store(false, core::sync::atomic::Ordering::Release);
}
